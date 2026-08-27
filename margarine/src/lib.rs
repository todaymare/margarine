pub mod resource;
#[doc(hidden)]
pub mod progress;
#[doc(hidden)]
pub mod version;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const VERSION_INFO: &str = concat!("margarine ", env!("CARGO_PKG_VERSION"));
pub const TARGET: &str = env!("MARGARINE_TARGET");


use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use progress::StatusLine;

use colourful::ColourBrush;
use common::string_map::StringIndex;
use errors::LexerError;
use errors::ParserError;
use errors::SemaError;
use git2::Repository;
pub use lexer::lex;
use parser::nodes::decl::Decl;
use parser::nodes::decl::DeclId;
use parser::nodes::decl::UseItem;
use parser::nodes::decl::UseItemKind;
use parser::nodes::decl::Visibility;
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
pub use semantic_analysis::llvm_codegen::{CompilationSettings, Prelude};
pub use semantic_analysis::llvm_codegen::CompilationTarget;
use sha2::Digest;
pub use semantic_analysis::{TyChecker};
pub use errors::display;
pub use sti::arena::Arena;
use sti::format_in;
use sti::vec::KVec;


pub use semantic_analysis;


pub struct Compiler<'me> {
    pub files: Files,
    pub arena: &'me Arena,
    pub string_map: StringMap<'me>,
    pub silent: bool,
    environment: HashMap<String, String>,
    build_depth: Option<usize>,
    repository_cache: Option<PathBuf>,
}


#[derive(Debug)]
pub enum CompilerError {
    BuildScript(BuildScriptError),
    PackageCopy {
        path: PathBuf,
        source: io::Error,
    },
    SourceOpen {
        path: PathBuf,
        source: io::Error,
    },
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    Link {
        target: CompilationTarget,
        status: Option<ExitStatus>,
        output: String,
    },
}


#[derive(Debug)]
pub enum BuildScriptError {
    Launch {
        path: PathBuf,
        source: io::Error,
    },
    Compile {
        path: PathBuf,
        source: Box<CompilerError>,
    },
    Failed {
        path: PathBuf,
        status: ExitStatus,
    },
}


impl fmt::Display for CompilerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildScript(error) => error.fmt(formatter),
            Self::PackageCopy { path, source } => {
                write!(formatter, "could not copy package to '{}': {source}", path.display())
            }
            Self::SourceOpen { path, source } => {
                write!(formatter, "could not open source '{}': {source}", path.display())
            }
            Self::Io { operation, path, source } => {
                write!(formatter, "could not {operation} '{}': {source}", path.display())
            }
            Self::Link { target, status, output } => {
                write!(formatter, "linking for target '{}' failed", target.margarine_target_triple())?;
                if let Some(status) = status {
                    write!(formatter, " with {status}")?;
                }
                for line in output.lines() {
                    write!(formatter, "\n  {line}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for CompilerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BuildScript(error) => Some(error),
            Self::PackageCopy { source, .. }
            | Self::SourceOpen { source, .. }
            | Self::Io { source, .. } => Some(source),
            Self::Link { .. } => None,
        }
    }
}

impl fmt::Display for BuildScriptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Launch { path, source } => {
                write!(formatter, "could not run build script '{}': {source}", path.display())
            }
            Self::Compile { path, source } => {
                write!(formatter, "could not compile build script '{}': {source}", path.display())
            }
            Self::Failed { path, status } => {
                write!(formatter, "build script '{}' failed with {status}", path.display())
            }
        }
    }
}

impl std::error::Error for BuildScriptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Launch { source, .. } => Some(source),
            Self::Compile { source, .. } => Some(source.as_ref()),
            Self::Failed { .. } => None,
        }
    }
}


impl From<BuildScriptError> for CompilerError {
    fn from(error: BuildScriptError) -> Self {
        Self::BuildScript(error)
    }
}



pub struct Files {
    files: Vec<FileData>,
}


#[allow(unused)]
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
            silent: false,
            environment: {
                let mut environment = current_environment();
                environment.remove("MARGARINE_COMPILATION_TARGET");
                environment
            },
            build_depth: None,
            repository_cache: None,
        }
    }



    pub fn check(&self, result: &mut CompilationResult<'me>) -> [Vec<Vec<String>>; 3] {
        let mut lex_error_files = Vec::with_capacity(result.errors.lexer_errors.len());
        for l in &result.errors.lexer_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let (report, primary_range) =
                display(e, &self.string_map, &self.files.files, &mut ());

                if !cfg!(feature="fuzzer")
                && !primary_range.is_some_and(|range| result.is_silent_range(range)) {
                    eprintln!("{report}");
                }

                file.push(report);
            }
            lex_error_files.push(file);
        }

        let mut parse_error_files = Vec::with_capacity(result.errors.parser_errors.len());
        for l in &result.errors.parser_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let (report, primary_range) =
                display(e, &self.string_map, &self.files.files, &mut ());

                if !cfg!(feature="fuzzer")
                && !primary_range.is_some_and(|range| result.is_silent_range(range)) {
                    eprintln!("{report}");
                }

                file.push(report);
            }
            parse_error_files.push(file);
        }

        let mut sema_errors = Vec::with_capacity(result.errors.sema_errors.len());
        for (id, error) in &result.errors.sema_errors {
            let (report, _) =
            display(error, &self.string_map, &self.files.files, &mut result.syms);

            if !cfg!(feature="fuzzer")
            && !result.is_silent_error(result.errors.sema_error_nodes[id]) {
                eprintln!("{report}");
            }

            sema_errors.push(report);
        }

        let errors = [lex_error_files, parse_error_files, vec![sema_errors]];
        errors
    }


    pub fn codegen(
        &mut self, 
        settings: &CompilationSettings, 
        result: &mut CompilationResult<'me>,
        errors: [Vec<Vec<String>>; 3]
    ) {
        result.codegen(self, &settings, settings.tests, errors);
    }


    pub fn run<'out>(
        &mut self, 
        settings: &CompilationSettings<'out>
    ) -> Result<CompilationResult<'out>, CompilerError> {
        let arena = settings.arena;
        let entry = self.string_map.insert(&settings.entry);
        let root_name = std::path::Path::new(&settings.entry)
            .file_stem()
            .map(|stem| stem.to_string_lossy())
            .filter(|stem| !stem.is_empty())
            .map(|stem| self.string_map.insert(&stem))
            .unwrap_or(entry);
        let preludes = settings.preludes
            .iter()
            .map(|p| (self.string_map.insert(&p.alias), self.string_map.insert(&p.url)))
            .collect::<Vec<_>>();

        let mut global = AST::new(&arena);
        let mut lex_errors = vec![];
        let mut parse_errors = vec![];

        let root = 
        global.add_decl(Decl::Module { 
            visibility: Visibility::Public,
            name: root_name, 
            header: SourceRange::ZERO, 
            body: Block::new(&[], SourceRange::ZERO), 
            is_root: true 
        }, SourceRange::ZERO);


        struct StackEntry {
            ast_node: DeclId,
            alias: StringIndex,
            path: StringIndex,
            visibility: Visibility,
            interpackage_depth: u32,
            is_included_by_prelude: bool,
        }

        let mut stack = vec![];
        let mut source_offset = 0;
        let mut counter = 0;

        let mut top_level : Vec<NodeId> = vec![root.into()];
        let mut top_modules = HashSet::new();

        let mut file_offsets = vec![];

        let mut state = CompilationState {
            linker_files: Vec::new(),
            packages: HashMap::new(),
        };
        let repository_cache =
            self.repository_cache.as_deref().unwrap_or(&settings.cache);

        let mut cfg_env = self
            .environment
            .iter()
            .map(|(key, value)| (self.string_map.insert(key), self.string_map.insert(value)))
            .collect::<HashMap<_, _>>();

        stack.push(StackEntry {
            ast_node: root,
            alias: root_name,
            path: entry,
            visibility: Visibility::Public,
            interpackage_depth: 0,
            is_included_by_prelude: false,
        });
        
        let comp_target = self.string_map.insert("MARGARINE_COMPILATION_TARGET");
        let target_triple = self.string_map.insert(&settings.compilation_target.llvm_target_triple());
        cfg_env.entry(comp_target).or_insert(target_triple);


        let package_depth = self.build_depth.unwrap_or_else(build_output_depth);

        while let Some(entry) = stack.pop() {
            let file_path = self.string_map.get(entry.path);
            let file = self.files.get(entry.path).unwrap();
            let depth = package_depth + entry.interpackage_depth as usize;

            let (tokens, le) = DropTimer::with_timer("tokenisation", || {
                let tokens = lex(&file, &mut self.string_map, source_offset);
                tokens
            });

            let (body, mut imports, mut pe) =
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
                    let using = global.add_decl(Decl::Using { visibility: Visibility::Private, item }, SourceRange::ZERO);
                    vec.push(import.into());
                    vec.push(using.into());
                    imports.push(import);
                }

                vec.extend_from_slice(&body);
                Block::new(vec.leak(), body.range())
            };



            file_offsets.push((entry.path, source_offset));
            source_offset += file.read().len() as u32;


            for (_, i) in imports {
                let source = global.range(i);
                match global.decl(i) {
                    Decl::ImportFile { name, visibility, .. } => {
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
                            visibility,
                            path: path_idx,
                            interpackage_depth: entry.interpackage_depth+1,
                            is_included_by_prelude: entry.is_included_by_prelude,
                        });
                    }


                    Decl::ImportRepo { alias, repo } => {
                        let repo_str = self.string_map.get(repo);

                        let package = load_package(
                            settings,
                            &repository_cache,
                            &mut state,
                            depth,
                            repo_str,
                            &self.environment,
                        );


                        let package =
                        match package {
                            Ok(p) => p,
                            Err(PackageError::RepoUnreachable) => {
                                let err = pe.push(parser::errors::Error::RepoDoesntExist {
                                    source,
                                    path: repo,
                                });
                                global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                                continue;
                            },

                            Err(PackageError::Compiler(error)) => return Err(error),
                        };

                        // Load lib.mar from the cloned repo
                        let lib_path = package.path.join("lib.mar");
                        let hash = package.resource.partial_string_hash;
                        let hash = self.string_map.insert(&hash);
                        let Ok(file) = FileData::open(&lib_path, &mut self.string_map)
                        else {
                            let lib_path_str = self.string_map.insert(&lib_path.to_string_lossy());
                            let err = pe.push(parser::errors::Error::FileDoesntExist { source, path: lib_path_str });
                            global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                            
                            continue;
                        };

                        let name = file.name();
                        let item = UseItem::new(
                            hash,
                            UseItemKind::BringName(alias),
                            source,
                        );
                        global.set_decl(
                            i,
                            Decl::Using {
                                visibility: Visibility::Private,
                                item,
                            },
                        );

                        if top_modules.contains(&hash) {
                            continue;
                        }

                        self.files.register(file);

                        let module = Decl::Module { 
                            visibility: Visibility::Private,
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
                            visibility: Visibility::Private,
                            interpackage_depth: 0,
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
                    visibility: entry.visibility,
                    name: entry.alias, 
                    header: offset, 
                    body, 
                    is_root: entry.interpackage_depth == 0,
                }
            );
            

            lex_errors.push(le);
            parse_errors.push(pe);

            counter += 1;

        }



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



        Ok(CompilationResult {
            file_offsets,

            errors: CompilationErrors {
                lexer_errors: lex_errors,
                parser_errors: parse_errors,
                sema_errors: sema.errors.errors,
                sema_error_nodes: sema.errors.nodes,
            },

            silent_ranges: merged_silent_ranges,

            tests,
            startups: sema.startups,
            ty_info: sema.type_info,
            scopes: sema.scopes,
            link_files: state.linker_files.into_iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            namespaces: sema.namespaces,
            syms: sema.syms,

            ast: global,
        })

}
}


fn current_environment() -> HashMap<String, String> {
    std::env::vars().collect()
}


pub fn preludes_from_env() -> Vec<Prelude> {
    let preludes = std::env::var("MARGARINE_PRELUDE")
        .iter()
        .flat_map(|value| value.split(';'))
        .filter_map(|value| value.split_once('='))
        .map(|(alias, url)| Prelude {
            alias: alias.into(),
            url: url.into(),
        })
        .collect::<Vec<_>>();

    if preludes.is_empty() {
        let url = format!("pkg:std");
        vec![Prelude { alias: "std".into(), url }]
    } else {
        preludes
    }
}

/// Compile and link a source file for the requested target.
pub fn build<P: AsRef<Path>, O: AsRef<Path>, C: AsRef<Path>>(
    path: P,
    target: CompilationTarget,
    output: O,
    cache: C,
    preludes: Vec<Prelude>,
) -> Result<PathBuf, CompilerError> {
    build_ex(path, target, output, cache, preludes, current_environment(), None, None)
}

fn build_ex<P: AsRef<Path>, O: AsRef<Path>, C: AsRef<Path>>(
    path: P,
    target: CompilationTarget,
    output: O,
    cache: C,
    preludes: Vec<Prelude>,
    environment: HashMap<String, String>,
    build_depth: Option<usize>,
    repository_cache: Option<PathBuf>,
) -> Result<PathBuf, CompilerError> {
    let path = path.as_ref();
    let output = output.as_ref().to_path_buf();
    let cache = cache.as_ref().to_path_buf();
    let mut environment = environment;
    if build_depth.is_none() {
        environment.remove("MARGARINE_COMPILATION_TARGET");
    }
    fs::create_dir_all(&cache)
        .map_err(|source| CompilerError::Io {
            operation: "create cache directory",
            path: cache.clone(),
            source,
        })?;

    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|source| CompilerError::Io {
                operation: "create output directory",
                path: parent.to_path_buf(),
                source,
            })?;
    }

    let arena = Arena::new();
    let mut compiler = Compiler::new(&arena);
    compiler.environment = environment;
    compiler.build_depth = build_depth;
    compiler.repository_cache = repository_cache;

    let file = FileData::open(path, &mut compiler.string_map)
        .map_err(|source| CompilerError::SourceOpen {
            path: path.to_path_buf(),
            source,
        })?;

    let entry = compiler.string_map.get(file.name()).into();
    compiler.files.register(file);

    let settings = CompilationSettings {
        compilation_target: target,
        preludes,
        entry,
        output: output.to_string_lossy().into_owned(),
        cache,
        arena: &arena,
        tests: false,
    };
    let compile_status =
    start_compilation_status(&settings, compiler.silent);


    let mut result = compiler.run(&settings)?;
    let errors =
    compile_status.suspend(|| compiler.check(&mut result));
    compiler.codegen(&settings, &mut result, errors);
    let mut link_files = result.link_files().to_vec();
    prepare_link_files(target, &mut link_files)?;

    link_compilation(target, &output, &link_files)?;
    compile_status.finish(format!("Built {}", path.display()));

    Ok(output)
}

pub fn prepare_link_files(
    target: CompilationTarget,
    link_files: &mut Vec<String>,
) -> Result<(), CompilerError> {
    let libs = resource::toolchain_libs_path(target);
    let files = resource::toolchain_link_files(target).map_err(|source| CompilerError::Io {
        operation: "read toolchain libraries",
        path: libs.clone(),
        source,
    })?;

    for path in files {
        let path = path.to_string_lossy().into_owned();
        if !link_files.contains(&path) {
            link_files.push(path);
        }
    }

    if let Some(missing) = link_files.iter().find(|path| !Path::new(path.as_str()).is_file()) {
        return Err(CompilerError::Link {
            target,
            status: None,
            output: format!("missing link input '{missing}'"),
        });
    }

    Ok(())
}



fn link_compilation(
    target: CompilationTarget,
    output: &Path,
    link_files: &[String],
) -> Result<(), CompilerError> {
    let toolchain_libs = resource::toolchain_libs_path(target);
    let output_object = format!("{}.o", output.display());

    let mut linker = match target {
        CompilationTarget::Arm64AppleDarwin => {
            let mut linker = Command::new("clang");
            linker
                .arg("-target")
                .arg(target.c_target_triple())
                .arg("-L")
                .arg(&toolchain_libs)
                .arg(&output_object)
                .args(link_files)
                .arg("-lzstd")
                .arg("-lz")
                .arg("-lc++")
                .arg("-lc++abi")
                .arg("-o")
                .arg(output);
            linker
        }
        CompilationTarget::X86_64UnknownLinuxGnu
        | CompilationTarget::Aarch64UnknownLinuxGnu => {
            let mut linker = Command::new("clang");
            linker
                .arg("-target")
                .arg(target.c_target_triple())
                .arg("-L")
                .arg(&toolchain_libs)
                .arg(&output_object)
                .args(link_files)
                .arg("-lzstd")
                .arg("-lz")
                .arg("-lstdc++")
                .arg("-o")
                .arg(output);
            linker
        }
        CompilationTarget::Wasm32UnknownUnknown => {
            let mut linker = Command::new("wasm-ld");
            linker
                .arg("--no-entry")
                .arg("--export=main")
                .arg("--export-memory")
                .arg("-L")
                .arg(&toolchain_libs)
                .arg(&output_object)
                .args(link_files)
                .arg("-o")
                .arg(output);
            linker
        }
    };

    let label =
    match target {
        CompilationTarget::Wasm32UnknownUnknown => "Linking browser Wasm",
        _ => "Linking",
    };
    let status = StatusLine::start(label);
    let result = linker.output().map_err(|source| CompilerError::Link {
        target,
        status: None,
        output: format!("could not start linker: {source}"),
    })?;
    if result.status.success() {
        status.finish(format!("Linked {}", output.display()));
        return Ok(());
    }

    let mut output_text = String::from_utf8_lossy(&result.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&result.stderr);
    if !stderr.is_empty() {
        if !output_text.is_empty() {
            output_text.push('\n');
        }
        output_text.push_str(&stderr);
    }
    Err(CompilerError::Link {
        target,
        status: Some(result.status),
        output: output_text,
    })
}


impl<'me> CompilationResult<'me> {
    pub fn codegen(
        &mut self,
        comp: &mut Compiler,
        settings: &CompilationSettings,
        tests: bool,
        errors: [Vec<Vec<String>>; 3],
    ) {
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
            settings,
        );

    }

    pub fn link_files(&self) -> &[String] { &self.link_files }

    /// Test symbol names paired with their `should_panic` flag.
    pub fn tests(&self) -> &[(SymbolId, bool)] { &self.tests }

    fn is_silent_error(&self, node: NodeId) -> bool {
        self.is_silent_range(self.ast.range(node))
    }

    fn is_silent_range(&self, source: SourceRange) -> bool {
        let (start, end) = source.range();
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


#[derive(Debug)]
enum PackageError {
    RepoUnreachable,
    Compiler(CompilerError),
}


const CACHE_REPOSITORY_DIR : &str = "repos";
const CACHE_BUILD_DIR : &str = "build";
const PACKAGE_SOURCE_DIR : &str = "package";
const PACKAGE_OUT_DIR : &str = "out";
const BUILD_SCRIPT_CACHE_DIR : &str = "cache";
const BUILD_SCRIPT_OUTPUT : &str = "build-script";


struct CompilationState {
    linker_files: Vec<PathBuf>,
    packages: HashMap<String, Package>,
}


#[derive(Clone)]
struct Package {
    resource: Resource,
    path: PathBuf,
}


fn load_package(
    settings: &CompilationSettings,
    repository_cache: &Path,
    state: &mut CompilationState,
    depth: usize,
    ident: &str,
    environment: &HashMap<String, String>,
) -> Result<Package, PackageError> {
    let url = resolve_url(ident);
    if let Some(package) = state.packages.get(&url) {
        return Ok(package.clone());
    }

    let resource = resource_cache_entry(repository_cache, &url)
        .map_err(PackageError::Compiler)?;

    if !load_repository(&resource.path, &url, resource.local) {
        return Err(PackageError::RepoUnreachable);
    }

    let build_root =
        settings.cache
            .join(CACHE_BUILD_DIR)
            .join(&resource.partial_string_hash);
    let path = build_root.join(PACKAGE_SOURCE_DIR);
    if build_root.exists() {
        std::fs::remove_dir_all(&build_root)
            .map_err(|source| PackageError::Compiler(CompilerError::PackageCopy {
                path: build_root.clone(),
                source,
            }))?;
    }
    copy_dir(&resource.path, &path)
        .map_err(|source| PackageError::Compiler(CompilerError::PackageCopy {
            path: path.clone(),
            source,
        }))?;

    let is_prelude = settings.preludes
        .iter()
        .any(|prelude| url == resolve_url(&prelude.url));

    let build_path = path.join("build.mar");
    if build_path.is_file() {
        if is_prelude {
            eprintln!(
                "{} build script inside a prelude package. skipped",
                "warning:".yellow().bold(),
            );
        } else {
            let links =
                run_build_script(
                    build_path,
                    &build_root,
                    repository_cache,
                    &url,
                    settings,
                    depth + 1,
                    environment,
                )
                    .map_err(|error| PackageError::Compiler(error.into()))?;
            state.linker_files.extend(links);
        }
    }

    let package = Package {
        resource,
        path,
    };
    state.packages.insert(url, package.clone());
    Ok(package)
}


fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_name() == ".git" {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}


fn load_repository(
    path: impl AsRef<Path>,
    url: &str,
    local: bool,
) -> bool {
    if local {
        return path.as_ref().is_dir();
    }
    if Repository::open(&path).is_ok() {
        return true;
    }

    // Missing or corrupted cache entry (e.g. leftovers
    // from an interrupted build): drop whatever is there
    // and clone afresh instead of poisoning the build.
    let _ = fs::remove_dir_all(&path);

    let status = StatusLine::start("Cloning");
    let cloned = Repository::clone(&url, &path).is_ok();
    if cloned {
        status.finish(format!("Cloned {}", url));
    }
    cloned
}

fn parse_build_script_stdout(stdout: &str, package_root: &Path) -> (Vec<PathBuf>, String) {
    let mut links = Vec::new();
    let mut output = String::new();
    for line in stdout.split_inclusive('\n') {
        let content = line.trim_end_matches(|character| matches!(character, '\r' | '\n'));
        if let Some(value) = content.strip_prefix("margarine:link=") {
            links.push(package_root.join(value));
        } else {
            output.push_str(line);
        }
    }
    (links, output)
}

fn write_build_script_output(
    mut writer: impl Write,
    stream: &str,
    source: &str,
    output: &str,
) -> io::Result<()> {
    if output.is_empty() {
        return Ok(());
    }

    writeln!(writer, "build script {stream} ({source}):")?;
    for line in output.split_inclusive('\n') {
        write!(writer, "  {line}")?;
    }
    if !output.ends_with('\n') {
        writeln!(writer)?;
    }
    Ok(())
}




fn run_build_script<P: AsRef<Path>>(
    build_path: P,
    build_root: &Path,
    repository_cache: &Path,
    source: &str,
    settings: &CompilationSettings<'_>,
    depth: usize,
    environment: &HashMap<String, String>,
) -> Result<Vec<PathBuf>, BuildScriptError> {
    let build_path = build_path.as_ref();
    let package_root = build_path.parent().unwrap_or_else(|| Path::new("."));
    let package_root = fs::canonicalize(package_root)
        .map_err(|source| BuildScriptError::Launch {
            path: build_path.to_path_buf(),
            source,
        })?;
    let build_root = fs::canonicalize(build_root)
        .map_err(|source| BuildScriptError::Launch {
            path: build_path.to_path_buf(),
            source,
        })?;
    let out_dir = build_root.join(PACKAGE_OUT_DIR);
    let cache_dir = build_root.join(BUILD_SCRIPT_CACHE_DIR);
    fs::create_dir_all(&out_dir)
        .and_then(|_| fs::create_dir_all(&cache_dir))
        .map_err(|source| BuildScriptError::Launch {
            path: build_path.to_path_buf(),
            source,
        })?;

    let output_path = build_root.join(BUILD_SCRIPT_OUTPUT);
    let prelude_env = settings
        .preludes
        .iter()
        .map(|prelude| format!("{}={}", prelude.alias, prelude.url))
        .collect::<Vec<_>>()
        .join(";");
    let requested_target = settings.compilation_target.margarine_target_triple().to_string();
    let package_dir = package_root.to_string_lossy().into_owned();
    let out_dir_string = format!("../{PACKAGE_OUT_DIR}");
    let mut build_env = environment.clone();
    build_env.insert("MARGARINE_BUILD_SCRIPT".to_string(), "1".to_string());
    build_env.insert("MARGARINE_BUILD_DEPTH".to_string(), depth.to_string());
    build_env.insert("MARGARINE_PRELUDE".to_string(), prelude_env);
    build_env.insert(
        "MARGARINE_COMPILATION_TARGET".to_string(),
        requested_target,
    );
    build_env.insert("MARGARINE_PACKAGE_DIR".to_string(), package_dir);
    build_env.insert("MARGARINE_OUT_DIR".to_string(), out_dir_string);

    build_ex(
        build_path,
        CompilationTarget::host(),
        &output_path,
        &cache_dir,
        settings.preludes.clone(),
        build_env.clone(),
        Some(depth),
        Some(repository_cache.to_path_buf()),
    ).map_err(|source| BuildScriptError::Compile {
        path: build_path.to_path_buf(),
        source: Box::new(source),
    })?;

    let running = StatusLine::start("Running build script");
    let mut command = Command::new(&output_path);
    command
        .current_dir(&package_root)
        .env_clear()
        .envs(&build_env)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let output =
    command.output()
        .map_err(|source| BuildScriptError::Launch {
            path: build_path.to_path_buf(),
            source,
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let (links, visible_stdout) = parse_build_script_stdout(stdout.as_ref(), &package_root);
    running.suspend(|| {
        let _ = write_build_script_output(io::stderr(), "stderr", source, &stderr);
        let _ = write_build_script_output(io::stdout(), "stdout", source, &visible_stdout);
    });
    running.clear();
    if !output.status.success() {
        return Err(BuildScriptError::Failed {
            path: build_path.to_path_buf(),
            status: output.status,
        });
    }

    Ok(links)
}


#[cfg(test)]
mod tests;



fn build_output_depth() -> usize {
    if std::env::var_os("MARGARINE_BUILD_SCRIPT").is_none() {
        return 0;
    }

    std::env::var("MARGARINE_BUILD_DEPTH")
        .ok()
        .and_then(|depth| depth.parse().ok())
        .unwrap_or(0)
}


#[doc(hidden)]
pub fn start_compilation_status(
    settings: &CompilationSettings,
    silent: bool,
) -> StatusLine {
    let entry = display_compile_path(settings, &settings.entry);
    StatusLine::start_compilation(format!("Compiling {}", entry), !silent)
}


fn display_compile_path(settings: &CompilationSettings, file_path: &str) -> String {
    let path = file_path.strip_prefix(&*settings.cache.to_string_lossy()).unwrap_or(file_path);
    let path = path.strip_prefix("/").unwrap_or(path);

    format!("{}.mar", path.replace("<>", "::"))
}




fn resolve_url(package: &str) -> String {
    if !package.starts_with("pkg:") {
        return package.to_string();
    }

    let default_base =
        format!("https://cdn.daymare.net/margarine/{VERSION}/share");
    let configured_base = if !cfg!(feature = "fuzzer") {
        std::env::var("MARGARINE_DEFAULT_URL").ok()
    } else {
        None
    };
    let base = configured_base.as_deref().unwrap_or(&default_base);

    let base = base.trim_end_matches('/');
    let package = package["pkg:".len()..].trim_matches('/');

    let resolved = format!("{base}/{package}");
    resolved
}


#[derive(Debug, Clone)]
struct Resource {
    partial_string_hash: String,
    path: PathBuf,
    local: bool,
}


fn resource_cache_entry(
    cache: &Path,
    ident: &str,
) -> Result<Resource, CompilerError> {
    let full_hash = sha2::Sha256::digest(ident.as_bytes());
    // Sixteen bytes produce the first thirty-two characters when hex-encoded.
    let string_hash = hex::encode(&full_hash[..16]);

    let local_source = Path::new(ident);
    let local = local_source.is_dir();
    let local_path =
    if local {
        local_source.to_path_buf()
    } else {
        let downloads_dir = cache.join(CACHE_REPOSITORY_DIR);
        std::fs::create_dir_all(&downloads_dir)
            .map_err(|source| CompilerError::Io {
                operation: "create package cache directory",
                path: downloads_dir.clone(),
                source,
            })?;
        downloads_dir.join(&string_hash)
    };
    Ok(Resource {
        partial_string_hash: string_hash,
        path: local_path,
        local,
    })
}
