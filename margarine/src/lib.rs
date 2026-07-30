use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use colourful::ColourBrush;
use common::string_map::StringIndex;
use errors::LexerError;
use errors::ParserError;
use errors::SemaError;
use git2::Repository;
pub use lexer::lex;
use parser::errors::Error;
use parser::nodes::decl::Decl;
use parser::nodes::decl::DeclId;
use parser::nodes::decl::UseItem;
use parser::nodes::decl::UseItemKind;
use parser::nodes::expr::Block;
use parser::nodes::NodeId;
use parser::nodes::AST;
pub use parser::parse;
pub use parser::nodes;
pub use common::source::{FileData, Extension};
pub use common::string_map::StringMap;
pub use common::{DropTimer, source::SourceRange};
use semantic_analysis::llvm_codegen;
use common::symbol_id::SymbolId;
pub use semantic_analysis::{TyChecker};
pub use errors::display;
use sha2::Digest;
use sha2::Sha256;
pub use sti::arena::Arena;
use sti::format_in;
use sti::vec::KVec;


pub use semantic_analysis;


pub struct Compiler<'me> {
    pub files: Files,
    pub arena: &'me Arena,
    pub string_map: StringMap<'me>,
    pub silent: bool,
}


pub struct Files {
    files: Vec<FileData>,
}


pub struct CompilationResult<'a> {
    file_offsets: Vec<(StringIndex, u32)>,
    pub errors: CompilationErrors,
    silent_ranges: Vec<SourceRange>,
    tests: Vec<(SymbolId, bool)>,
    ast: AST<'a>,
    startups: KVec<u32, SymbolId>,
    ty_info: semantic_analysis::TyInfo<'a>,
    pub syms: semantic_analysis::syms::sym_map::SymbolMap<'a>,
    namespaces: semantic_analysis::namespace::NamespaceMap,
    scopes: semantic_analysis::scope::ScopeMap<'a>,
    link_files: Vec<String>,
}


#[derive(Debug)]
pub struct CompilationErrors {
    pub lexer_errors : Vec<KVec<LexerError , lexer::errors::Error>>,
    pub parser_errors: Vec<KVec<ParserError, parser::errors::Error>>,
    pub sema_errors  : KVec<SemaError  , semantic_analysis::errors::Error>,
    pub sema_error_nodes: KVec<SemaError, NodeId>,
}


#[derive(Clone)]
pub struct CompilationSettings<'out> {
    pub compilation_target: CompilationTarget,
    pub preludes: Vec<Prelude>,
    pub entry: String,
    pub output: &'out Arena,
    pub tests: bool,
}


#[derive(Clone, Copy)]
pub enum CompilationTarget {
    Arm64AppleDarwin,
    Wasm32UnknownUnknown,
}


#[derive(Clone)]
pub struct Prelude {
    pub alias: String,
    pub url: String,
}


impl<T: AsRef<str>> From<T> for CompilationTarget {
    fn from(value: T) -> Self {
        match value.as_ref() {
            "default" | "arm64-apple-arch" => CompilationTarget::Arm64AppleDarwin,
            "wasm32-unknown-unknown" => CompilationTarget::Wasm32UnknownUnknown,
            value => {
                eprintln!("unsupported compilation target: {value}");
                std::process::abort();
            }
        }
    }
}


impl<'me> Compiler<'me> {
    pub fn new(arena: &'me Arena) -> Self {
        Self {
            files: Files { files: vec![] },
            arena,
            string_map: StringMap::new(arena),
            silent: true,
        }
    }


    pub fn run<'out>(
        &mut self, 
        settings: &CompilationSettings<'out>
    ) -> CompilationResult<'out> {
        println!("{}", settings.entry);

        let arena = settings.output;
        let entry = self.string_map.insert(&settings.entry);
        let preludes = settings.preludes
            .iter()
            .map(|p| (self.string_map.insert(&p.alias), self.string_map.insert(&p.url)))
            .collect::<Vec<_>>();

        let mut global = AST::new(&arena);
        let mut lex_errors = vec![];
        let mut parse_errors = vec![];
        let mut build_lock = BuildLock::load();


        let root = 
        global.add_decl(Decl::Module { 
            name: entry, 
            header: SourceRange::ZERO, 
            body: Block::new(&[], SourceRange::ZERO), 
            is_root: true 
        }, SourceRange::ZERO);


        struct StackEntry {
            ast_node: DeclId,
            alias: StringIndex,
            path: StringIndex,
            intercrate_depth: u32,
            is_included_by_prelude: bool,
        }

        let mut stack = vec![];
        let mut source_offset = 0;
        let mut counter = 0;

        let mut top_level : Vec<NodeId> = vec![root.into()];
        let mut top_modules = HashSet::new();

        let mut file_offsets = vec![];
        let mut package_urls: HashMap<String, String> = HashMap::new();
        let mut link_file_paths = HashMap::new();
        let mut cfg_env = std::env::vars()
            .map(|(k, v)| (self.string_map.insert(&k), self.string_map.insert(&v)))
            .collect::<HashMap<_, _>>();

        stack.push(StackEntry {
            ast_node: root,
            alias: entry,
            path: entry,
            intercrate_depth: 0,
            is_included_by_prelude: false,
        });
        
        let comp_target = self.string_map.insert("MARGARINE_COMPILATION_TARGET");
        let target_triple = self.string_map.insert(&llvm_api::ctx::package_target_triple());
        cfg_env.insert(comp_target, target_triple);


        while let Some(entry) = stack.pop() {
            let file_path = self.string_map.get(entry.path);
            let file = self.files.get(entry.path).unwrap();
            let depth = entry.intercrate_depth as usize;

            if !self.silent {
                let display = display_compile_path(file_path, &package_urls);
                if depth != 0 {
                    println!(
                        "{}{}{} {} {}",
                        "|".dark_grey(),
                        "-".repeat(depth).dark_grey(),
                        ">".dark_grey(),
                        "compiling:".green().bold(),
                        display,
                    );
                } else {
                    println!(
                        "{} {}",
                        "compiling:".green().bold(),
                        display,
                    );
                }
            }



            let (tokens, le) = DropTimer::with_timer("tokenisation", || {
                let tokens = lex(&file, &mut self.string_map, source_offset);
                tokens
            });

            let (body, mut imports, link_files, mut pe) = 
            DropTimer::with_timer("parsing", || {
                parse(tokens, counter, &arena, &mut self.string_map, &mut global, &cfg_env)
            });

            let body = 
            if entry.is_included_by_prelude { body }
            else {
                let mut vec = sti::vec::Vec::with_cap_in(arena, preludes.len() * 2 + body.len());
                
                for &p in &preludes {
                    let import = global.add_decl(Decl::ImportRepo { alias: p.0, repo: p.1 }, SourceRange::ZERO);
                    let item = UseItem::new(StringMap::PRELUDE, UseItemKind::All, SourceRange::ZERO);
                    let item = UseItem::new(p.0, UseItemKind::List { list: arena.alloc_new([item]) }, SourceRange::ZERO);
                    let using = global.add_decl(Decl::Using { item }, SourceRange::ZERO);
                    vec.push(import.into());
                    vec.push(using.into());
                    imports.push(import);
                }

                vec.extend_from_slice(&body);
                Block::new(vec.leak(), body.range())
            };

            for (_, link_file) in link_files {
                let Decl::LinkFile { url, hash } = global.decl(link_file) 
                else { unreachable!() };
                let source = global.range(link_file);

                let url_str = self.string_map.get(url);
                let url = resolve_url(url_str);
                let resource = resource_cache_entry(&url);

                let (tempfile, file_hash) = 
                if resource.path.is_file() {
                    let Ok(file) = std::fs::read(&resource.path)
                    else {
                        let reason = self.string_map.insert("unable to open cached resource file");
                        let url = self.string_map.insert(&url);
                        let err = pe.push(Error::ExternalFileError { source, url, operation: "check resource hash", reason });
                        global.set_decl(link_file, Decl::Error(errors::ErrorId::Parser((counter, err))));
                        continue;
                    };

                    (None, Sha256::digest(file).try_into().unwrap())

                } else {

                    let Some(cache_dir) = resource.path.parent() 
                    else {
                        let reason = self.string_map.insert("unable to determine the resource cache directory");
                        let url = self.string_map.insert(&url);
                        let err = pe.push(Error::ExternalFileError { source, url, operation: "prepare resource cache", reason });
                        global.set_decl(link_file, Decl::Error(errors::ErrorId::Parser((counter, err))));
                        continue;
                    };

                    let mut tempfile = 
                    match tempfile::Builder::new().tempfile_in(cache_dir) {
                        Ok(file) => file,
                        Err(error) => {
                            let reason = self.string_map.insert(&error.to_string());
                            let url = self.string_map.insert(&url);
                            let err = pe.push(Error::ExternalFileError { source, url, operation: "prepare resource cache", reason });
                            global.set_decl(link_file, Decl::Error(errors::ErrorId::Parser((counter, err))));
                            continue;
                        }
                    };

                    if !self.silent {
                        println!("{}{}{} {} {}", "|".dark_grey(), "-".repeat(depth+1).dark_grey(), ">".dark_grey(), "downloading...".green().bold(), url);
                    }


                    let hash = 
                    match download_and_hash(&url, tempfile.as_file_mut()) {
                        Ok(hash) => hash,
                        Err(error) => {
                            let reason = self.string_map.insert(&error.to_string());
                            let url = self.string_map.insert(&url);
                            let err = pe.push(Error::ExternalFileError { 
                                source, 
                                url, 
                                operation: "download external file", 
                                reason 
                            });

                            global.set_decl(
                                link_file, 
                                Decl::Error(errors::ErrorId::Parser((counter, err)))
                            );
                            continue;
                        }
                    };

                    (Some(tempfile), hash)
                };

                if let Some((hash, source_hash)) = hash {
                    let hash_str = self.string_map.get(hash);

                    let Ok(expected_hash) = hex::decode(hash_str) 
                    else {
                        let err = pe.push(Error::InvalidHash { source: source_hash });
                        global.set_decl(link_file, Decl::Error(errors::ErrorId::Parser((counter, err))));
                        continue;
                    };

                    if expected_hash != file_hash {
                        let found_hash = hex::encode(file_hash);
                        let found_hash = self.string_map.insert(&found_hash);
                        let err = pe.push(Error::HashMismatch { 
                            source_extern: source, 
                            source_hash,
                            expected: hash,
                            actual: found_hash,
                        });

                        global.set_decl(
                            link_file, 
                            Decl::Error(errors::ErrorId::Parser((counter, err)))
                        );
                        continue
                    }
                }

                if let Some(Err(error)) = tempfile.map(|t| t.persist(&resource.path)) {
                    let reason = self.string_map.insert(&error.error.to_string());
                    let url = self.string_map.insert(&url);
                    let err = pe.push(Error::ExternalFileError { source, url, operation: "store external file", reason });
                    global.set_decl(link_file, Decl::Error(errors::ErrorId::Parser((counter, err))));
                    continue;
                }


                link_file_paths.insert(
                    link_file, 
                    resource.path.to_string_lossy().into_owned()
                );
            }


            file_offsets.push((entry.path, source_offset));
            source_offset += file.read().len() as u32;


            for (_, i) in imports {
                let source = global.range(i);
                match global.decl(i) {
                    Decl::ImportFile { name, .. } => {
                        let path = format_in!(&arena, "{}/{}.mar", file_path, self.string_map.get(name));

                        let path_idx = self.string_map.insert(
                            &std::path::Path::new(&*path).with_extension("").to_string_lossy()
                        );

                        if self.files.get(path_idx).is_none() {
                            let Ok(file) = FileData::open(&*path, &mut self.string_map)
                            else {
                                let path_str = format_in!(&arena, "{}/{}", file_path, self.string_map.get(name));
                                let path_idx = self.string_map.insert(&path_str);
                                let err = pe.push(parser::errors::Error::FileDoesntExist { source, path: path_idx });
                                global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                                
                                continue;
                            };

                            self.files.register(file);
                        }

                        stack.push(StackEntry {
                            ast_node: i,
                            alias: name,
                            path: path_idx,
                            intercrate_depth: entry.intercrate_depth+1,
                            is_included_by_prelude: entry.is_included_by_prelude,
                        });
                    }


                    Decl::ImportRepo { alias, repo } => {
                        let repo_str = self.string_map.get(repo);
                        let alias_str = self.string_map.get(alias);
                        let url = resolve_url(repo_str);

                        let resource = resource_cache_entry(&url);

                        package_urls.insert(
                            resource.partial_string_hash.clone(), 
                            url.clone()
                        );

                        let hash = self.string_map.insert(&resource.partial_string_hash);

                        {
                            let item = UseItem::new(
                                hash, 
                                UseItemKind::BringName(alias), 
                                source
                            );

                            global.set_decl(
                                i, 
                                Decl::Using { item }
                            );
                        }

                        let repository = 
                        if std::fs::exists(&resource.path).unwrap_or(false) {
                            match Repository::open(&resource.path) {
                                Ok(repo) => repo,
                                Err(_) => {
                                    let err = pe.push(parser::errors::Error::RepoDoesntExist {
                                        source,
                                        path: repo,
                                    });
                                    global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                                    continue;
                                }
                            }
                        } else {
                            if !self.silent {
                                println!("{}{}{} {} {}", "|".dark_grey(), "-".repeat(depth+1).dark_grey(), ">".dark_grey(), "downloading...".green().bold(), url);
                            }

                            match Repository::clone(&url, &resource.path) {
                                Ok(repo) => repo,
                                Err(_) => {
                                    let err = pe.push(parser::errors::Error::RepoDoesntExist {
                                        source,
                                        path: repo,
                                    });
                                    global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                                    continue;
                                }
                            }
                        };

                        let target_commit = "HEAD";

                        let Ok(object) = repository.revparse_single(&target_commit) else {
                            let err = pe.push(parser::errors::Error::RepoDoesntExist {
                                source,
                                path: repo,
                            });
                            global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                            continue;
                        };

                        if repository.checkout_tree(&object, None).is_err()
                        || repository.set_head_detached(object.id()).is_err()
                        {
                            let err = pe.push(parser::errors::Error::RepoDoesntExist {
                                source,
                                path: repo,
                            });
                            global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                            continue;
                        }

                        let commit = object.id().to_string();
                        build_lock.set(alias_str.to_string(), commit);

                        // if the module is already in the top level, skip it
                        if top_modules.contains(&hash) {
                            continue;
                        }

                        // Load lib.mar from the cloned repo
                        let lib_path = resource.path.join("lib.mar");
                        let Ok(file) = FileData::open(&lib_path, &mut self.string_map)
                        else {
                            let lib_path_str = self.string_map.insert(&lib_path.to_string_lossy());
                            let err = pe.push(parser::errors::Error::FileDoesntExist { source, path: lib_path_str });
                            global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                            
                            continue;
                        };

                        let name = file.name();
                        self.files.register(file);

                        let module = Decl::Module { 
                            name: hash,
                            header: source, 
                            body: Block::new(&[], SourceRange::ZERO), 
                            is_root: true,
                        };

                        let module = global.add_decl(module, SourceRange::ZERO);
                        top_level.push(module.into());
                        top_modules.insert(hash);

                        let is_prelude = preludes.iter().find(|n| n.1 == repo).is_some();
                        stack.insert(0, StackEntry { 
                            ast_node: module, 
                            alias: hash, 
                            path: name, 
                            intercrate_depth: 0, 
                            is_included_by_prelude: is_prelude
                        });

                    }


                    _ => unreachable!()
                }
            }


            let offset = global.range(entry.ast_node);
            global.set_decl(
                entry.ast_node, 
                Decl::Module { 
                    name: entry.alias, 
                    header: offset, 
                    body, 
                    is_root: entry.intercrate_depth == 0,
                }
            );
            

            lex_errors.push(le);
            parse_errors.push(pe);

            counter += 1;

        }

        let _ = build_lock.save();

        self.files.sort_by(&file_offsets);

        let temp = Arena::new();
        let sema = {
            let _1 = DropTimer::new("semantic analysis");
            TyChecker::run(&arena, &temp, &mut global, &top_level, &mut self.string_map)
        };


        let tests: Vec<(SymbolId, bool)> = sema.tests.iter().copied().collect();
        let mut silent_ranges = sema.silent_ranges;
        silent_ranges.sort_unstable_by_key(|range| range.range());
        let mut merged_silent_ranges: Vec<SourceRange> = Vec::with_capacity(silent_ranges.len());
        for range in silent_ranges {
            let (start, end) = range.range();
            if let Some(last) = merged_silent_ranges.last_mut() {
                let (last_start, last_end) = last.range();
                if start <= last_end {
                    *last = SourceRange::new(last_start, last_end.max(end));
                    continue;
                }
            }
            merged_silent_ranges.push(range);
        }


        let mut link_files = Vec::new();
        let mut seen_link_files = HashSet::new();
        for (_, link_file) in &sema.link_files {
            let path = link_file_paths.get(link_file).unwrap();
            if seen_link_files.insert(path.clone()) {
                link_files.push(path.clone());
            }
        }

        CompilationResult {
            file_offsets,

            errors: CompilationErrors {
                lexer_errors: lex_errors,
                parser_errors: parse_errors,
                sema_errors: sema.errors,
                sema_error_nodes: sema.error_nodes,
            },

            silent_ranges: merged_silent_ranges,

            tests,
            startups: sema.startups,
            ty_info: sema.type_info,
            scopes: sema.scopes,
            link_files,
            namespaces: sema.namespaces,
            syms: sema.syms,

            ast: global,
        }
    }

}


impl<'me> CompilationResult<'me> {
    pub fn codegen(&mut self, comp: &mut Compiler, tests: bool, errors: [Vec<Vec<String>>; 3]) {
        let tests = 
        if tests { self.tests.iter().map(|s| s.0).collect() } 
        else { vec![] };

        llvm_codegen::run(
            &mut comp.string_map, &mut self.syms,
            &mut self.namespaces, &mut self.ast,
            &mut self.ty_info, errors,
            self.file_offsets.len() as u32,
            &self.startups,
            &tests,
        );

    }

    pub fn link_files(&self) -> &[String] { &self.link_files }

    fn is_silent_error(&self, node: NodeId) -> bool {
        let (start, end) = self.ast.range(node).range();
        let index = self.silent_ranges.partition_point(|range| range.range().0 <= start);
        index != 0 && end <= self.silent_ranges[index - 1].range().1
    }
}


impl Files {
    pub fn register(&mut self, fd: FileData) {
        if let Some(file) = self.get_mut(fd.name()) {
            *file = fd;
        } else {
            self.files.push(fd);
        };
    }


    pub fn get(&self, name: StringIndex) -> Option<&FileData> {
        self.files.iter().find(|x| x.name() == name)
    }


    fn get_mut(&mut self, name: StringIndex) -> Option<&mut FileData> {
        self.files.iter_mut().find(|x| x.name() == name)
    }


    pub fn files(&self) -> &[FileData] {
        &self.files
    }


    pub fn sort_by(&mut self, offsets: &[(StringIndex, u32)]) {
        self.files.sort_by_key(|f| offsets.iter().find(|n| n.0 == f.name()).map(|x| x.1).unwrap_or(u32::MAX));
    }
}



pub fn run<'str>(mut settings: CompilationSettings) -> (Vec<String>, Vec<(String, bool)>) {
    let mut comp = Compiler::new(settings.output);
    comp.silent = false;

    let file = FileData::open(&settings.entry, &mut comp.string_map).unwrap();
    settings.entry = comp.string_map.get(file.name()).into();
    comp.files.register(file);

    let mut result = comp.run(&settings);

    let mut lex_error_files = Vec::with_capacity(result.errors.lexer_errors.len());
    for l in &result.errors.lexer_errors {
        let mut file = Vec::with_capacity(l.len());
        for e in l.iter() {
            let report = display(e, &comp.string_map, &comp.files.files, &mut ());

            if !cfg!(feature="fuzzer") {
                println!("{report}");
            }

            file.push(report);
        }
        lex_error_files.push(file);
    }

    let mut parse_error_files = Vec::with_capacity(result.errors.parser_errors.len());
    for l in &result.errors.parser_errors {
        let mut file = Vec::with_capacity(l.len());
        for e in l.iter() {
            let report = display(e, &comp.string_map, &comp.files.files, &mut ());

            if !cfg!(feature="fuzzer") {
                println!("{report}");
            }

            file.push(report);
        }
        parse_error_files.push(file);
    }

    let mut sema_errors = Vec::with_capacity(result.errors.sema_errors.len());
    for (id, error) in &result.errors.sema_errors {
        let report = display(error, &comp.string_map, &comp.files.files, &mut result.syms);

        if !cfg!(feature="fuzzer")
        && !result.is_silent_error(result.errors.sema_error_nodes[id]) {
            println!("{report}");
        }

        sema_errors.push(report);
    }

    let errors = [lex_error_files, parse_error_files, vec![sema_errors]];

    result.codegen(&mut comp, settings.tests, errors);
    let link_files = result.link_files().to_vec();

    let mut tests = vec![];
    for (sym, should_panic) in &result.tests {
        tests.push((comp.string_map.get(result.syms.sym(*sym).name()).to_string(), *should_panic));
    }

    (link_files, tests)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_link_file_is_a_parser_error() {
        let arena = Arena::new();
        let mut compiler = Compiler::new(&arena);
        let name = compiler.string_map.insert("test.mar");
        compiler.files.register(FileData::new(
            "extern \"missing-linker-input.o\";\n\
             @cfg(env(\"PATH\", \"__margarine_cfg_disabled__\"))\n\
             extern \"also-missing.o\";"
                .to_string(),
            name,
            Extension::None,
        ));

        let result = compiler.run(&arena, name);
        let errors: Vec<_> = result.errors.parser_errors.iter().flatten().collect();

        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].1, parser::errors::Error::ExternalFileError { .. }));
    }

    #[test]
    fn conditional_trait_impl_requires_its_generic_bounds() {
        let arena = Arena::new();
        let mut compiler = Compiler::new(&arena);
        let name = compiler.string_map.insert("test.mar");
        compiler.files.register(FileData::new(
            "trait Printable {}\n\
             impl Printable for int {}\n\
             impl<T0: Printable, T1: Printable> Printable for (T0, T1) {}\n\
             struct Wrapper<T> { value: T }\n\
             impl<T: Printable> Printable for Wrapper<T> {}\n\
             fn requires_printable<T: Printable>(value: T) {}\n\
             fn main() {\n\
                 requires_printable((1, 2));\n\
                 requires_printable(Wrapper { value: (1, false) });\n\
             }"
                .to_string(),
            name,
            Extension::None,
        ));

        let result = compiler.run(&arena, name);
        let errors: Vec<_> = result.errors.sema_errors.iter().collect();

        assert_eq!(errors.iter().filter(|error| matches!(
            error,
            semantic_analysis::errors::Error::TypeDoesntImplTrait { .. }
        )).count(), 1);
    }
}


fn display_compile_path(file_path: &str, package_urls: &HashMap<String, String>) -> String {
    let path = file_path.strip_prefix("artifacts/").unwrap_or(file_path);
    let mut parts = path.split('/');
    let Some(first) = parts.next() else {
        return format!("{}.mar", file_path.replace("<>", "::"));
    };

    if let Some(url) = package_urls.get(first) {
        let rest: Vec<&str> = parts.collect();
        if rest.is_empty() || rest == ["lib"] {
            return url.clone();
        }
        return format!("/{}.mar", rest.join("/"));
    }

    format!("{}.mar", path.replace("<>", "::"))
}


struct BuildLock {
    packages: HashMap<String, String>, // alias -> commit hash
}


impl BuildLock {
    fn load() -> Self {
        match fs::read_to_string("build.lock") {
            Ok(content) => {
                let mut lock = BuildLock { packages: HashMap::new() };

                for line in content.lines() {
                    let (name, commit) = line.split_once(",").unwrap();
                    lock.packages.insert(name.to_string(), commit.to_string());
                }

                lock
            }

            Err(_) => BuildLock { packages: HashMap::new() },
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let mut content = String::new();
        for (alias, commit) in &self.packages {
            sti::write!(&mut content, "{},{}\n", alias, commit);
        }

        fs::write("build.lock", content)
    }

    fn get(&self, alias: &str) -> Option<String> {
        self.packages.get(alias).cloned()
    }

    fn set(&mut self, alias: String, commit: String) {
        self.packages.insert(alias, commit);
    }
}


fn resolve_url(url: &str) -> String {
    if !url.starts_with("pkg:") {
        url.to_string()
    } else {
        let base =
        if !cfg!(feature="fuzzer") { std::env::var("MARGARINE_DEFAULT_URL").ok() }
        else { None };

        let base = base.as_ref().map(|x| x.as_str())
            .unwrap_or("https://pkg.daymare.net/margarine");

        let base = base.trim_end_matches('/');
        let url = url.trim_start_matches('/');
        let url = &url["pkg:".len()..];

        format!("{base}/{url}")
    }
}


struct Resource {
    full_hash: [u8; 32],
    partial_string_hash: String,
    path: PathBuf,
}


fn resource_cache_entry(ident: &str) -> Resource {
    let full_hash = sha2::Sha256::digest(ident.as_bytes());
    let partial_hash = u128::from_be_bytes(full_hash[..16].try_into().unwrap());
    let string_hash = format!("{partial_hash:032}");

    let artifacts_dir = PathBuf::from("artifacts");
    std::fs::create_dir_all(&artifacts_dir).unwrap();

    let local_path = artifacts_dir.join(&string_hash);
    Resource {
        full_hash: full_hash[..].try_into().unwrap(),
        partial_string_hash: string_hash,
        path: local_path,
    }
}


fn download_and_hash(
    url: &str,
    file: &mut File,
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let mut response = reqwest::blocking::get(url)?.error_for_status()?;
    let mut hasher = Sha256::new();

    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = response.read(&mut buf)?;
        if n == 0 {
            break;
        }

        let chunk = &buf[..n];

        file.write_all(chunk)?;
        hasher.update(chunk);
    }

    Ok(hasher.finalize().into())
}
