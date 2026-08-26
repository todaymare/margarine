use super::*;

struct EnvGuard {
    name: &'static str,
    value: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn new(name: &'static str) -> Self {
        Self { name, value: std::env::var_os(name) }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.value {
            std::env::set_var(self.name, value);
        } else {
            std::env::remove_var(self.name);
        }
    }
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

    let result = compiler.run(&CompilationSettings {
        compilation_target: CompilationTarget::Arm64AppleDarwin,
        preludes: vec![],
        entry: "test.mar".to_string(),
        arena: &arena,
        tests: false,
        output: "program".to_string(),
        cache: "artifacts".into(),
    }).unwrap();
    let errors: Vec<_> = result.errors.sema_errors.iter().collect();

    assert_eq!(errors.iter().filter(|error| matches!(
        error,
        semantic_analysis::errors::Error::TypeDoesntImplTrait { .. }
    )).count(), 1);
}

fn compile_source(source: &str) -> CompilationResult<'_> {
    let arena = Box::leak(Box::new(Arena::new()));
    let mut compiler = Compiler::new(arena);
    let name = compiler.string_map.insert("test.mar");
    compiler.files.register(FileData::new(source.to_owned(), name, Extension::None));
    compiler.run(&CompilationSettings {
        compilation_target: CompilationTarget::Arm64AppleDarwin,
        preludes: vec![],
        entry: "test.mar".to_string(),
        arena,
        tests: false,
        output: "program".to_string(),
        cache: "artifacts".into(),
    }).unwrap()
}

#[test]
fn silent_attributes_cover_lexer_and_parser_diagnostics() {
    let source =
        "@test(should_panic)\n\
         @silent\n\
         fn invalid_numbers() {\n\
             1e9999999999;\n\
             assert(0b102 == 0, \"unreachable\");\n\
         }";
    let arena = Arena::new();
    let mut compiler = Compiler::new(&arena);
    let name = compiler.string_map.insert("test.mar");
    compiler.files.register(FileData::new(source.to_owned(), name, Extension::None));
    let result = compiler.run(&CompilationSettings {
        compilation_target: CompilationTarget::Arm64AppleDarwin,
        preludes: vec![],
        entry: "test.mar".to_string(),
        arena: &arena,
        tests: false,
        output: "program".to_string(),
        cache: "artifacts".into(),
    }).unwrap();
    let lexer_errors = result.errors.lexer_errors
        .iter()
        .flat_map(|errors| errors.iter())
        .collect::<Vec<_>>();
    let parser_errors = result.errors.parser_errors
        .iter()
        .flat_map(|errors| errors.iter())
        .collect::<Vec<_>>();

    assert!(!lexer_errors.is_empty());
    assert!(!parser_errors.is_empty());
    assert!(lexer_errors.iter().all(|error| {
        let (_, range) =
        display(*error, &compiler.string_map, &compiler.files.files, &mut ());
        range.is_some_and(|range| result.is_silent_range(range))
    }));
    assert!(parser_errors.iter().all(|error| {
        let (_, range) =
        display(*error, &compiler.string_map, &compiler.files.files, &mut ());
        range.is_some_and(|range| result.is_silent_range(range))
    }));
}

#[test]
fn build_script_launch_errors_preserve_their_source() {
    let source = io::Error::new(io::ErrorKind::Other, "spawn failed");
    let error = CompilerError::from(BuildScriptError::Launch {
        path: PathBuf::from("package/build.mar"),
        source,
    });

    assert_eq!(
        error.to_string(),
        "could not run build script 'package/build.mar': spawn failed",
    );
    let build_script_error = std::error::Error::source(&error)
        .expect("compiler error should retain the build-script error");
    assert_eq!(
        std::error::Error::source(build_script_error)
            .expect("build-script error should retain the launch cause")
            .to_string(),
        "spawn failed",
    );
}

#[cfg(unix)]
#[test]
fn build_script_exit_errors_report_status() {
    use std::os::unix::process::ExitStatusExt;

    let error = CompilerError::from(BuildScriptError::Failed {
        path: PathBuf::from("package/build.mar"),
        status: std::process::ExitStatus::from_raw(7 << 8),
    });

    assert_eq!(
        error.to_string(),
        "build script 'package/build.mar' failed with exit status: 7",
    );
}

#[test]
fn build_script_stdout_preserves_output_and_link_order() {
    let package = Path::new("/package");
    let stdout = "starting native build\n\
margarine:link=lib/first.a\r\n\
running user command\n\
margarine:link=/tmp/second.a\n\
margarine:link=lib/first.a";

    let (links, visible) = parse_build_script_stdout(stdout, package);

    assert_eq!(links, [
        PathBuf::from("/package/lib/first.a"),
        PathBuf::from("/tmp/second.a"),
        PathBuf::from("/package/lib/first.a"),
    ]);
    assert_eq!(visible, "starting native build\nrunning user command\n");
}

#[test]
fn linker_error_details_are_indented_under_the_summary() {
    let error = CompilerError::Link {
        target: CompilationTarget::Arm64AppleDarwin,
        status: None,
        output: "first detail\nsecond detail\n".into(),
    };

    assert_eq!(
        error.to_string(),
        "linking for target 'arm64-apple-darwin' failed\n  first detail\n  second detail",
    );
}

#[test]
fn build_script_output_is_labeled_and_indented() {
    let cases = [
        ("", ""),
        ("one", "build script stderr (https://example.com/pkg.git):\n  one\n"),
        ("one\n", "build script stderr (https://example.com/pkg.git):\n  one\n"),
        (
            "one\n\nthree",
            "build script stderr (https://example.com/pkg.git):\n  one\n  \n  three\n",
        ),
    ];

    for (output, expected) in cases {
        let mut rendered = Vec::new();
        write_build_script_output(&mut rendered, "stderr", "https://example.com/pkg.git", output).unwrap();
        assert_eq!(String::from_utf8(rendered).unwrap(), expected);
    }
}

#[test]
fn prelude_environment_has_a_std_default_and_preserves_explicit_order() {
    let _guard = EnvGuard::new("MARGARINE_PRELUDE");
    std::env::remove_var("MARGARINE_PRELUDE");

    let defaults = preludes_from_env();
    assert_eq!(defaults.len(), 1);
    assert_eq!(defaults[0].alias, "std");
    let expected =
    resource::development_library_root()
        .map(|root| root.join("std").to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("https://cdn.daymare.net/margarine/{VERSION}/std"));
    assert_eq!(defaults[0].url, expected);
    std::env::set_var(
        "MARGARINE_PRELUDE",
        "first=https://example.com/first;second=https://example.com/second",
    );
    let explicit = preludes_from_env();
    assert_eq!(
        explicit.iter().map(|prelude| prelude.alias.as_str()).collect::<Vec<_>>(),
        ["first", "second"],
    );
    assert_eq!(explicit[0].url, "https://example.com/first");
    assert_eq!(explicit[1].url, "https://example.com/second");
}



#[test]
fn package_artifacts_are_grouped_by_hash_and_rebuilt_cleanly() {
    let workspace = tempfile::tempdir().unwrap();
    let source = workspace.path().join("source");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("lib.mar"), "pub fn value(): int { 1 }").unwrap();

    let repository = Repository::init(&source).unwrap();
    let mut index = repository.index().unwrap();
    index.add_path(Path::new("lib.mar")).unwrap();
    let tree = repository.find_tree(index.write_tree().unwrap()).unwrap();
    let signature = git2::Signature::now("Margarine", "margarine@localhost").unwrap();
    repository.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "fixture",
        &tree,
        &[],
    ).unwrap();
    drop(tree);
    drop(repository);

    let arena = Arena::new();
    let cache = workspace.path().join("cache");
    let settings = CompilationSettings {
        compilation_target: CompilationTarget::host(),
        preludes: vec![],
        entry: "unused.mar".to_string(),
        output: "unused".to_string(),
        cache: cache.clone(),
        arena: &arena,
        tests: false,
    };
    let mut state = CompilationState {
        linker_files: Vec::new(),
        packages: HashMap::new(),
    };
    let environment = HashMap::new();
    let url = source.to_string_lossy();

    let first = load_package(&settings, &cache, &mut state, 0, &url, &environment).unwrap();
    let hash = &first.resource.partial_string_hash;
    assert_eq!(first.resource.path, source);
    assert_eq!(first.path, cache.join(CACHE_BUILD_DIR).join(hash).join(PACKAGE_SOURCE_DIR));
    assert!(!first.path.join(".git").exists());
    let marker = first.path.join("materialization-marker");
    fs::write(&marker, "kept").unwrap();
    let build_root = first.path.parent().unwrap();
    let out_dir = build_root.join(PACKAGE_OUT_DIR);
    fs::create_dir(&out_dir).unwrap();
    fs::write(out_dir.join("stale"), "stale").unwrap();

    let second = load_package(&settings, &cache, &mut state, 0, &url, &environment).unwrap();
    assert_eq!(first.path, second.path);
    assert_eq!(fs::read_to_string(marker).unwrap(), "kept");
    assert!(out_dir.join("stale").exists());
    assert_eq!(state.packages.len(), 1);

    let mut next_compilation = CompilationState {
        linker_files: Vec::new(),
        packages: HashMap::new(),
    };
    let rebuilt =
        load_package(&settings, &cache, &mut next_compilation, 0, &url, &environment).unwrap();
    assert_eq!(rebuilt.path, first.path);
    assert!(!rebuilt.path.join("materialization-marker").exists());
    assert!(!out_dir.join("stale").exists());
    assert!(!rebuilt.path.join(".git").exists());
}

#[test]
fn build_scripts_share_repository_cache_and_keep_artifacts_together() {
    let workspace = tempfile::tempdir().unwrap();
    let workspace_root = workspace.path().join("workspace with spaces");
    let dependency_source = workspace_root.join("dependency");
    fs::create_dir_all(&dependency_source).unwrap();
    fs::write(dependency_source.join("lib.mar"), "pub fn value(): int { 1 }").unwrap();
    let repository = Repository::init(&dependency_source).unwrap();
    let mut index = repository.index().unwrap();
    index.add_path(Path::new("lib.mar")).unwrap();
    let tree = repository.find_tree(index.write_tree().unwrap()).unwrap();
    let signature = git2::Signature::now("Margarine", "margarine@localhost").unwrap();
    repository.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "fixture",
        &tree,
        &[],
    ).unwrap();
    drop(tree);
    drop(repository);
    let build_root = workspace_root.join(CACHE_BUILD_DIR).join("package-hash");
    let package_root = build_root.join(PACKAGE_SOURCE_DIR);
    fs::create_dir_all(&package_root).unwrap();
    let build_path = package_root.join("build.mar");
    fs::write(
        &build_path,
        format!(
            r#"
import "{}" as dependency;

@cfg(env("MARGARINE_OUT_DIR", "../out"))
fn main() {{}}

@cfg(not(env("MARGARINE_OUT_DIR", "../out")))
fn main() {{
    output_directory_must_be_relative;
}}
"#,
            dependency_source.display(),
        ),
    ).unwrap();

    let arena = Arena::new();
    let settings = CompilationSettings {
        compilation_target: CompilationTarget::host(),
        preludes: vec![],
        entry: "unused.mar".to_string(),
        output: "unused".to_string(),
        cache: workspace.path().join("unused-cache"),
        arena: &arena,
        tests: false,
    };
    let repository_cache = workspace.path().join("shared-cache");

    let links =
        run_build_script(
            &build_path,
            &build_root,
            &repository_cache,
            "fixture",
            &settings,
            1,
            &HashMap::new(),
        ).unwrap();

    assert!(links.is_empty());
    assert!(build_root.join(BUILD_SCRIPT_OUTPUT).is_file());
    assert!(build_root.join(BUILD_SCRIPT_CACHE_DIR).is_dir());
    assert!(build_root.join(PACKAGE_OUT_DIR).is_dir());
    assert!(!package_root.join(".margarine-out").exists());
    let dependency = resource_cache_entry(
        &repository_cache,
        &dependency_source.to_string_lossy(),
    ).unwrap();
    assert!(dependency.path.is_dir());
    assert!(!build_root
        .join(BUILD_SCRIPT_CACHE_DIR)
        .join(CACHE_REPOSITORY_DIR)
        .exists());
}

#[test]
fn package_cache_creation_errors_are_typed() {
    let workspace = tempfile::tempdir().unwrap();
    let cache = workspace.path().join("not-a-directory");
    fs::write(&cache, "file").unwrap();
    let arena = Arena::new();
    let settings = CompilationSettings {
        compilation_target: CompilationTarget::host(),
        preludes: vec![],
        entry: "unused.mar".to_string(),
        output: "unused".to_string(),
        cache,
        arena: &arena,
        tests: false,
    };

    let error = resource_cache_entry(&settings.cache, "fixture")
        .expect_err("a file cannot contain the package cache directory");

    assert!(matches!(error, CompilerError::Io {
        operation: "create package cache directory",
        ..
    }));
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn compile_paths_stay_relative_to_the_cache() {
    let arena = Arena::new();
    let settings = CompilationSettings {
        compilation_target: CompilationTarget::host(),
        preludes: vec![],
        entry: "unused.mar".to_string(),
        output: "unused".to_string(),
        cache: "/tmp/cache".into(),
        arena: &arena,
        tests: false,
    };

    assert_eq!(
        display_compile_path(&settings, "/tmp/cache/deps/package/lib"),
        "deps/package/lib.mar",
    );
}

#[test]
fn package_copy_errors_preserve_their_source() {
    let workspace = tempfile::tempdir().unwrap();
    let missing_source = workspace.path().join("missing-package");
    let destination = workspace.path().join("materialized-package");
    let source = copy_dir(&missing_source, &destination)
        .expect_err("copying a missing package should fail");
    let error = CompilerError::PackageCopy {
        path: destination,
        source,
    };

    assert!(
        error
            .to_string()
            .starts_with("could not copy package to '"),
    );
    assert_eq!(
        std::error::Error::source(&error)
            .expect("package copy error should retain the I/O error")
            .downcast_ref::<io::Error>()
            .expect("package copy source should be an I/O error")
            .kind(),
        io::ErrorKind::NotFound,
    );
}



#[test]
fn expression_type_info_covers_propagated_errors() {
    let result = compile_source("fn main() { var value = missing + 1; }");
    assert!(result.errors.sema_errors.iter().any(|error| matches!(
        error,
        semantic_analysis::errors::Error::VariableNotFound { .. }
    )));

    assert!(
        result.ty_info.exprs.iter().all(|info| info.is_some()),
        "every parsed expression must receive type information"
    );
}

#[test]
fn semantic_errors_do_not_panic_codegen() {
    let result = compile_source(
        "mod math { struct Vec3 { value: int } \
         impl Vec3 { fn new(value: int): Self { Self { value } } } } \
         type Color = math::Vec3; \
         fn main() { var color = Color::new(1); }",
    );

    assert!(result.errors.sema_errors.iter().any(|error| matches!(
        error,
        semantic_analysis::errors::Error::PrivateSymbol { .. }
    )));
    assert!(
        result.ty_info.exprs.iter().any(|info| info.is_some()),
        "failed namespace expressions must be recorded"
    );
}

#[test]
fn private_symbols_cannot_be_imported_by_a_sibling_module() {
    let result = compile_source("mod a { fn secret() {} } mod b { use a::secret }");
    assert!(result.errors.sema_errors.iter().any(|error| matches!(error, semantic_analysis::errors::Error::PrivateSymbol { .. })));
}

#[test]
fn public_symbols_can_be_imported_and_reexported() {
    let result = compile_source("mod a { pub fn exposed() {} } mod b { pub use a::exposed } use b::exposed");
    assert!(!result.errors.sema_errors.iter().any(|error| matches!(error, semantic_analysis::errors::Error::PrivateSymbol { .. })));
}

#[test]
fn reimporting_the_same_symbol_is_idempotent() {
    let result = compile_source("mod a { pub fn exposed() {} } use a::exposed use a::exposed");
    assert!(!result.errors.sema_errors.iter().any(|error| matches!(error, semantic_analysis::errors::Error::NameIsAlreadyDefined { .. })));
}

#[test]
fn ambiguous_trait_methods_are_reported() {
    let result = compile_source(
        "trait First { fn method(self) }\n\
         trait Second { fn method(self) }\n\
         struct Value {}\n\
         impl First for Value { fn method(self) {} }\n\
         impl Second for Value { fn method(self) {} }\n\
         fn main() { Value {}.method() }",
    );
    assert!(result.errors.sema_errors.iter().any(|error| matches!(
        error,
        semantic_analysis::errors::Error::AmbiguousTraitMethod { .. }
    )));
}
#[test]
fn captured_closure_values_cannot_be_mutated() {
    let cases = [
        (
            "direct assignment",
            "fn main() {\n\
                 var captured = 1;\n\
                 var closure = || { captured = 2; };\n\
             }",
        ),
        (
            "compound assignment",
            "fn main() {\n\
                 var captured = 1;\n\
                 var closure = || { captured += 1; };\n\
             }",
        ),
        (
            "nested field assignment",
            "struct Value { field: int }\n\
             struct Holder { value: Value }\n\
             fn main() {\n\
                 var holder = Holder { value: Value { field: 1 } };\n\
                 var closure = || { holder.value.field = 2; };\n\
             }",
        ),
        (
            "mutation through &",
            "fn increment(&value: int) { value += 1; }\n\
             fn main() {\n\
                 var values = [1, 2];\n\
                 var closure = || { increment(&values[0]); };\n\
             }",
        ),
        (
            "unwrap assignment",
            "struct Optional { value: Option<int> }\n\
             fn main() {\n\
                 var optional = Optional { value: some(1) };\n\
                 var closure = || { optional.value! = 2; };\n\
             }",
        ),
        (
            "or-return assignment",
            "struct Optional { value: Option<int> }\n\
             fn main() {\n\
                 var optional = Optional { value: some(1) };\n\
                 var closure = || { optional.value? = 2; };\n\
             }",
        ),
    ];

    for (name, source) in cases {
        let result = compile_source(source);
        assert_eq!(
            result.errors.sema_errors.iter().filter(|error| matches!(
                error,
                semantic_analysis::errors::Error::CannotMutateCapturedValue { .. }
            )).count(),
            1,
            "{name}",
        );
    }
}

#[test]
fn captured_closure_unwrap_assignment_cannot_be_mutated() {
    let result = compile_source(
        "struct Optional { value: Option<int> }\n\
         fn main() {\n\
             var optional = Optional { value: some(1) };\n\
             var closure = || { optional.value! = 2; };\n\
         }",
    );

    assert_eq!(result.errors.sema_errors.iter().filter(|error| matches!(
        error,
        semantic_analysis::errors::Error::CannotMutateCapturedValue { .. }
    )).count(), 1);
}

#[test]
fn closure_local_values_remain_mutable_and_captures_remain_readable() {
    let result = compile_source(
        "fn main() {\n\
             var captured = 1;\n\
             var read = || { captured };\n\
             var local = || { var value = 1; value += 1; value };\n\
         }",
    );

    assert!(!result.errors.sema_errors.iter().any(|error| matches!(
        error,
        semantic_analysis::errors::Error::CannotMutateCapturedValue { .. }
    )));
}
