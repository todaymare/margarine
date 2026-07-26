#![feature(if_let_guard)]
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use colourful::ColourBrush;
use common::string_map;
use common::string_map::StringIndex;
use errors::LexerError;
use errors::ParserError;
use errors::SemaError;
use git2::Repository;
pub use lexer::lex;
use parser::nodes::decl::Decl;
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
pub use sti::arena::Arena;
use sti::format_in;
use sti::vec::KVec;
use tracing::trace;


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


impl<'me> Compiler<'me> {
    pub fn new(arena: &'me Arena) -> Self {
        Self {
            files: Files { files: vec![] },
            arena,
            string_map: StringMap::new(arena),
            silent: true,
        }
    }


    pub fn run<'out>(&mut self, arena: &'out Arena, entry: StringIndex) -> CompilationResult<'out> {
        tracing::trace!("compiling program. entry point is '{}'", self.string_map.get(entry));

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


        let mut stack = vec![(root, entry, entry, true, 0)];
        let mut source_offset = 0;
        let mut counter = 0;

        let mut top_level : Vec<NodeId> = vec![root.into()];
        let mut top_modules = HashSet::new();

        let mut file_offsets = vec![];
        let mut package_urls: HashMap<String, String> = HashMap::new();
        let mut link_file_paths: HashMap<parser::nodes::decl::DeclId, String> = HashMap::new();
        let mut cfg_env = std::env::vars()
            .map(|(k, v)| (self.string_map.insert(&k), self.string_map.insert(&v)))
            .collect::<HashMap<_, _>>();
        
        let comp_target = self.string_map.insert("COMPILATION_TARGET");
        let target_triple = self.string_map.insert(&llvm_api::ctx::default_target_triple());
        cfg_env.insert(comp_target, target_triple);

        while let Some((decl, name, path, is_root, depth)) = stack.pop() {
            let file_path = self.string_map.get(path);

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


            let file = self.files.get(path).unwrap();

            let (tokens, le) = DropTimer::with_timer("tokenisation", || {
                let tokens = lex(&file, &mut self.string_map, source_offset);
                tokens
            });

            let (body, imports, link_files, mut pe) = 
            DropTimer::with_timer("parsing", || {
                parse(tokens, counter, &arena, &mut self.string_map, &mut global, &cfg_env)
            });

            for (_, link_file) in link_files {
                let Decl::LinkFile { path } = global.decl(link_file) 
                else { unreachable!() };
                let path = Path::new(file_path)
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .join(self.string_map.get(path));
                if !fs::exists(&path).unwrap_or(false) {
                    let path = self.string_map.insert(&path.to_string_lossy());
                    let err = pe.push(parser::errors::Error::FileDoesntExist {
                        source: global.range(link_file),
                        path,
                    });
                    global.set_decl(link_file, Decl::Error(errors::ErrorId::Parser((counter, err))));
                    continue;
                }
                link_file_paths.insert(link_file, path.to_string_lossy().into_owned());
            }


            file_offsets.push((path, source_offset));
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

                        stack.push((i, name, path_idx, false, depth+1));
                    }


                    Decl::ImportRepo { alias, repo } => {
                        let repo_str = self.string_map.get(repo);
                        let (url, commit) = if repo_str.contains('@') {
                            let parts: Vec<_> = repo_str.splitn(2, '@').collect();
                            (parts[0], Some(parts[1]))
                        } else {
                            (repo_str, None)
                        };



                        let alias_str = self.string_map.get(alias);

                        let url =
                        if !url.starts_with("pkg:") {
                            url.to_string()
                        } else {
                            let base =
                            if !cfg!(feature="fuzzer") { std::env::var("MARGARINE_DEFAULT_URL").ok() }
                            else { None };

                            let base = base.as_ref().map(|x| x.as_str()).unwrap_or("https://pkg.daymare.net/margarine");
                            let base = base.trim_end_matches('/');
                            let url = url.trim_start_matches('/');
                            let url = &url["pkg:".len()..];

                            format!("{base}/{url}")
                        };


                        let dir_hash = sha2::Sha256::digest(url.as_bytes());
                        let dir_hash = u64::from_be_bytes(dir_hash[..8].try_into().unwrap());
                        let dir_hash = format!("{:016x}", dir_hash);

                        package_urls.insert(dir_hash.clone(), url.clone());

                        let hash = self.string_map.insert(&dir_hash);

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

                        let artifacts_dir = "artifacts";
                        if !std::fs::exists(artifacts_dir).unwrap_or(false) {
                            std::fs::create_dir(artifacts_dir).unwrap();
                        }

                        let local_path = format!("{}/{}", artifacts_dir, dir_hash);

                        let repository = if std::fs::exists(&local_path).unwrap_or(false) {
                            match Repository::open(&local_path) {
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

                            match Repository::clone(&url, &local_path) {
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

                        let target_commit = if let Some(commit) = commit {
                            commit.to_string()
                        } else if let Some(lock) = build_lock.get(&alias_str) {
                            lock
                        } else {
                            match repository.head() {
                                Ok(head) => head.target()
                                    .map(|oid| oid.to_string())
                                    .unwrap_or_else(|| "HEAD".to_string()),
                                Err(_) => "HEAD".to_string(),
                            }
                        };

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

                        build_lock.set(alias_str.to_string(), target_commit);

                        // if the module is already in the top level, skip it
                        if top_modules.contains(&hash) {
                            continue;
                        }

                        // Load lib.mar from the cloned repo
                        let lib_path = format!("{}/lib.mar", local_path);
                        let Ok(file) = FileData::open(&lib_path, &mut self.string_map)
                        else {
                            let lib_path_str = self.string_map.insert(&lib_path);
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

                        stack.insert(0, (module, hash, name, true, 0));

                    }


                    _ => unreachable!()
                }
            }


            let offset = global.range(decl);

            global.set_decl(
                decl, 
                Decl::Module { 
                    name, 
                    header: offset, 
                    body, 
                    is_root
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

    fn report_errors(&self, errors: &[Vec<Vec<String>>; 3]) {
        for files in [&errors[0], &errors[1]] {
            for file in files {
                for error in file {
                    println!("{error}");
                }
            }
        }

        for ((id, _), error) in (&self.errors.sema_errors).into_iter().zip(&errors[2][0]) {
            if !self.is_silent_error(self.errors.sema_error_nodes[id]) {
                println!("{error}");
            }
        }
    }

    fn build_errors(&mut self, comp: &mut Compiler) -> [Vec<Vec<String>>; 3] {
        let mut lex_error_files = Vec::with_capacity(self.errors.lexer_errors.len());
        for l in &self.errors.lexer_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let report = display(e, &comp.string_map, &comp.files.files, &mut ());
                file.push(report);
            }
            lex_error_files.push(file);
        }

        let mut parse_error_files = Vec::with_capacity(self.errors.parser_errors.len());
        for l in &self.errors.parser_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let report = display(e, &comp.string_map, &comp.files.files, &mut ());
                file.push(report);
            }
            parse_error_files.push(file);
        }

        let mut sema_errors = Vec::with_capacity(self.errors.sema_errors.len());
        for (id, error) in &self.errors.sema_errors {
            let report = display(error, &comp.string_map, &comp.files.files, &mut self.syms);
            sema_errors.push(report);
        }

        [lex_error_files, parse_error_files, vec![sema_errors]]
    }

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



pub fn run<'str>(string_map: StringMap, files: FileData, tests: bool) -> (Vec<String>, Vec<(String, bool)>) {
    let name = files.name();
    let arena = string_map.arena();
    let mut comp = Compiler::new(&arena);
    comp.string_map = string_map;
    comp.silent = false;
    comp.files.register(files);

    let mut result = comp.run(&arena, name);
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

    result.codegen(&mut comp, tests, errors);
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
        assert!(matches!(errors[0].1, parser::errors::Error::FileDoesntExist { .. }));
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
