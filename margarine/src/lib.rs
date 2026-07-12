#![feature(if_let_guard)]
use std::collections::HashMap;
use std::fs;

use colourful::ColourBrush;
use common::string_map::StringIndex;
use errors::LexerError;
use errors::ParserError;
use errors::SemaError;
use git2::Repository;
pub use lexer::lex;
use parser::nodes::decl::Decl;
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
    tests: Vec<SymbolId>,
    ast: AST<'a>,
    startups: KVec<u32, SymbolId>,
    ty_info: semantic_analysis::TyInfo<'a>,
    pub syms: semantic_analysis::syms::sym_map::SymbolMap<'a>,
    namespaces: semantic_analysis::namespace::NamespaceMap,
    scopes: semantic_analysis::scope::ScopeMap<'a>,
}


#[derive(Debug)]
pub struct CompilationErrors {
    pub lexer_errors : Vec<KVec<LexerError , lexer::errors::Error>>,
    pub parser_errors: Vec<KVec<ParserError, parser::errors::Error>>,
    pub sema_errors  : KVec<SemaError  , semantic_analysis::errors::Error>,
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


        let mut stack = vec![(None, entry, 0)];
        let mut source_offset = 0;
        let mut counter = 0;

        let mut root = None;

        let mut file_offsets = vec![];

        while let Some((decl, f, depth)) = stack.pop() {
            let file_path = self.string_map.get(f);

            if !self.silent {
                if depth != 0 {
                    let name =
                    if file_path.starts_with("artifacts/") {
                        &file_path["artifacts/".len()..]
                    } else { &file_path };

                    println!(
                        "{}{}{} {} {}.mar", 
                        "|".dark_grey(), 
                        "-".repeat(depth).dark_grey(), 
                        ">".dark_grey(), 
                        "compiling:".green().bold(),
                        name.replace("<>", "::")
                    );
                } else {
                    println!(
                        "{} {}.mar",
                        "compiling:".green().bold(),
                        file_path,
                    );
                }

            }


            let file = self.files.get(f).unwrap();

            let (tokens, le) = DropTimer::with_timer("tokenisation", || {
                let tokens = lex(&file, &mut self.string_map, source_offset);
                tokens
            });

            let (body, imports, mut pe) = DropTimer::with_timer("parsing", || {
                parse(tokens, counter, &arena, &mut self.string_map, &mut global)
            });


            file_offsets.push((f, source_offset));
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

                        stack.push((Some((i, name)), path_idx, depth+1));
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

                        let mut hasher = sti::hash::fxhash::FxHasher64::new();
                        hasher.write_bytes(url.as_bytes());
                        let dir_hash = format!("{:016x}", hasher.hash);

                        let artifacts_dir = "artifacts";
                        if !std::fs::exists(artifacts_dir).unwrap_or(false) {
                            std::fs::create_dir(artifacts_dir).unwrap();
                        }

                        let local_path = format!("{}/{}", artifacts_dir, dir_hash);

                        if !std::fs::exists(&local_path).unwrap_or(false) {
                            if !self.silent {
                                println!("{}{}{} {} {}", "|".dark_grey(), "-".repeat(depth+1).dark_grey(), ">".dark_grey(), "downloading...".green().bold(), url);
                            }

                            let repo =
                            match Repository::clone(&url, &local_path) {
                                Ok(v) => v,
                                Err(_) => {
                                    let err = pe.push(parser::errors::Error::RepoDoesntExist { 
                                        source, 
                                        path: repo 
                                    });
                                    global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                                    continue;
                                },
                            };


                            let target_commit =
                            if let Some(commit) = commit {
                                commit.to_string()
                            } else if let Some(lock) = build_lock.get(&alias_str) {
                                lock
                            } else {
                                // Get HEAD commit
                                match repo.head() {
                                    Ok(head) => {
                                        head.target()
                                            .map(|oid| oid.to_string())
                                            .unwrap_or_else(|| "HEAD".to_string())
                                    }
                                    Err(_) => "HEAD".to_string(),
                                }
                            };

                            // Checkout the commit
                            if let Ok(obj) = repo.revparse_single(&target_commit) {
                                let _ = repo.checkout_tree(&obj, None);
                            }


                            // Update lock file
                            build_lock.set(alias_str.to_string(), target_commit);

                        } else if let Some(commit) = commit {
                            // If repo exists but user specified a commit, checkout it
                            if let Ok(repo) = Repository::open(&local_path) {
                                if let Ok(obj) = repo.revparse_single(commit) {
                                    let _ = repo.checkout_tree(&obj, None);
                                }
                                build_lock.set(alias_str.to_string(), commit.to_string());
                            }
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

                        stack.push((Some((i, alias)), name, depth+1));
                    }


                    _ => unreachable!()
                }
            }

            if let Some((decl, name)) = decl {
                let offset = global.range(decl);
                global.set_decl(decl, Decl::Module { name, header: offset, body, user_defined: true });
            } else {
                root = Some(body);
            }
            

            lex_errors.push(le);
            parse_errors.push(pe);

            counter += 1;

        }

        let _ = build_lock.save();

        self.files.sort_by(&file_offsets);

        // todo: find a way to comrpess these errors into vecs
        for l in &lex_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let report = display(e, &self.string_map, &self.files.files, &mut ());
                #[cfg(not(feature = "fuzzer"))]
                println!("{report}");
                file.push(report);
            }
        }

        for l in &parse_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let report = display(e, &self.string_map, &self.files.files, &mut ());
                #[cfg(not(feature = "fuzzer"))]
                println!("{report}");
                file.push(report);
            }
        }

        let temp = Arena::new();
        let sema = {
            let _1 = DropTimer::new("semantic analysis");
            TyChecker::run(&arena, &temp, &mut global, &root.unwrap(), &mut self.string_map)
        };


        let mut tests = Vec::with_capacity(sema.startups.len());

        for t in sema.tests {
            tests.push(t.1);
        }


        CompilationResult {
            file_offsets,

            errors: CompilationErrors {
                lexer_errors: lex_errors,
                parser_errors: parse_errors,
                sema_errors: sema.errors,
            },

            tests,
            startups: sema.startups,
            ty_info: sema.type_info,
            scopes: sema.scopes,
            namespaces: sema.namespaces,
            syms: sema.syms,

            ast: global,
        }
    }

}


impl<'me> CompilationResult<'me> {
    pub fn codegen(&mut self, comp: &mut Compiler) -> Vec<u8> {

        // todo: find a way to comrpess these errors into vecs
        let mut lex_error_files = Vec::with_capacity(self.errors.lexer_errors.len());
        for l in &self.errors.lexer_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let report = display(e, &comp.string_map, &comp.files.files, &mut ());
                #[cfg(not(feature = "fuzzer"))]
                println!("{report}");
                file.push(report);
            }

            lex_error_files.push(file);
        }

        let mut parse_error_files = Vec::with_capacity(self.errors.parser_errors.len());
        for l in &self.errors.parser_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let report = display(e, &comp.string_map, &comp.files.files, &mut ());
                #[cfg(not(feature = "fuzzer"))]
                println!("{report}");
                file.push(report);
            }

            parse_error_files.push(file);
        }

        let mut sema_errors = Vec::with_capacity(self.errors.sema_errors.len());
        for s in &self.errors.sema_errors {
            let report = display(s.1, &comp.string_map, &comp.files.files, &mut self.syms);

            #[cfg(not(feature = "fuzzer"))]
            println!("{report}");

            sema_errors.push(report);
        } 

        llvm_codegen::run(
            &mut comp.string_map, &mut self.syms, 
            &mut self.namespaces, &mut self.ast,
            &mut self.ty_info, [lex_error_files, parse_error_files, vec![sema_errors]],
            self.file_offsets.len() as u32,
            &self.startups
        );

        vec![]
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



pub fn run<'str>(string_map: StringMap, files: FileData) -> (Vec<u8>, Vec<String>) {
    let name = files.name();
    let arena = string_map.arena();
    let mut comp = Compiler::new(&arena);
    comp.string_map = string_map;
    comp.silent = false;
    comp.files.register(files);

    let mut result = comp.run(&arena, name);
    let src = result.codegen(&mut comp);

    let mut tests = vec![];
    for test in &result.tests {
        tests.push(comp.string_map.get(result.syms.sym(*test).name()).to_string());
    }

    (src, tests)
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

