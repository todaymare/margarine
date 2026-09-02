use std::{path::{Path, PathBuf}, process::{Command, Stdio}};

use colourful::ColourBrush;
use margarine::{start_compilation_status, BuildMode, CompilationSettings, CompilationTarget};
use sti::arena::Arena;

use super::{CliError, CliResult, COMPILE_ERROR, PROGRAM_ERROR, TICK_GLYPH};

/// Compiles `path` with the Compiler + CompilationResult pipeline and links
/// the result for `target`. Shared by `build` and `run`.
pub(super) fn compile_and_link(
    path: &Path,
    target: CompilationTarget,
    mode: BuildMode,
    output: Option<String>,
    cache: String,
) -> CliResult<String> {

    let output_suffix =
    if matches!(mode, BuildMode::Shared) {
        target.shared_library_suffix().to_string()
    } else {
        target.output_suffix()
    };

    let output = output
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let path = path.with_extension("");
            let name = path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or("program".into());
            PathBuf::from(&cache)
                .join(name)
                .with_extension(output_suffix)
        });
    let current_dir =
        std::env::current_dir()
            .map_err(|error| CliError::link(format!("cannot get current directory: {error}")))?;
    let output = if output.is_absolute() {
        output
    } else {
        current_dir.join(output)
    };
    let cache = if Path::new(&cache).is_absolute() {
        PathBuf::from(&cache)
    } else {
        current_dir.join(&cache)
    };
    margarine::build(
        path,
        target,
        &output,
        &cache,
        margarine::preludes_from_env(),
        mode,
    ).map_err(|error| CliError::link(format!("build failed: {error}")))?;


    Ok(output.to_string_lossy().into_owned())
}

pub(super) fn run(
    path: &Path,
    target: CompilationTarget,
    output: Option<String>,
    cache: String,
    program_args: Vec<String>,
) -> CliResult<i32> {
    let output = compile_and_link(path, target, BuildMode::Executable, output, cache)?;

    println!("{}", format!("› Running {}", path.display()).dim());

    let status = Command::new(&output)
        .args(program_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| CliError::link(format!("cannot run '{output}': {error}")))?;

    if status.success() {
        Ok(0)
    } else {
        let code = status.code().unwrap_or(PROGRAM_ERROR);
        Ok(if (0..=125).contains(&code) { code } else { PROGRAM_ERROR })
    }
}

pub(super) fn check(
    path: &Path,
    target: CompilationTarget,
    cache: String,
) -> CliResult<i32> {
    let arena = Arena::new();
    let mut compiler = margarine::Compiler::new(&arena);
    let file = margarine::FileData::open(
        &path.display().to_string(),
        &mut compiler.string_map,
    ).map_err(|error| CliError::link(format!("cannot open '{}': {error}", path.display())))?;
    let entry = compiler.string_map.get(file.name()).into();
    compiler.files.register(file);

    let settings = CompilationSettings {
        compilation_target: target,
        preludes: margarine::preludes_from_env(),
        entry,
        output: String::new(),
        cache: cache.into(),
        arena: &arena,
        tests: false,
        shared: false,
    };
    let compile_status = start_compilation_status(&settings, compiler.silent);

    let mut result =
        match compiler.run(&settings) {
            Ok(result) => result,
            Err(error) => {
                compile_status.clear();
                return Err(CliError::link(format!("compilation failed: {error}")));
            },
        };
    let errors = compile_status.suspend(|| compiler.check(&mut result));
    compile_status.clear();
    let error_count = errors.iter().flatten().map(|file| file.len()).sum::<usize>();

    if error_count == 0 {
        println!("{} no errors found", TICK_GLYPH.green());
        Ok(0)
    } else {
        Ok(COMPILE_ERROR)
    }
}
