use std::{ffi::CString, fmt::Write, io::{self, Write as _}, path::PathBuf, process::Command, time::Instant};

use clap::{Parser, Subcommand};
use colourful::ColourBrush;
use margarine::{CompilationSettings, CompilationTarget, Prelude};
use sti::{arena::Arena};

#[derive(Parser)]
#[command(
    name = "margarine",
    version,
    propagate_version = true,
    about = "a unified development toolchain for the Margarine programming language",
    after_help = format!(
        "{}\n  {} {} {}\n  {} {}\n\n{} {}",
        "Quick start:".bold().underline(),
        "margarine run".bold(),
        "hello.mar".cyan(),
        "# compile & execute".dim(),
        "margarine test".bold(),
        "tests/core.mar".cyan(),
        "docs:".dim(),
        "https://github.com/todaymare/margarine",
    ),
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Library management
    Lib {
        #[command(subcommand)]
        command: LibCommands,
    },

    /// Compile a source file into an executable
    Build {
        /// Source path
        #[arg(value_parser = existing_file_path)]
        path: PathBuf,

        /// Compilation target
        #[arg(long, default_value = "default")]
        target: CompilationTarget,

        /// Output path
        #[arg(short, long)]
        output: Option<String>,

        /// Cache directory
        #[arg(long)]
        cache: Option<String>,

        /// Reset the build cache before compiling
        #[arg(long)]
        update: bool,
    },

    /// Compile and run a source file
    Run {
        /// Source path
        #[arg(value_parser = existing_file_path)]
        path: PathBuf,

        /// Cache directory
        #[arg(long)]
        cache: Option<String>,

        /// Reset the build cache before compiling
        #[arg(long)]
        update: bool,

        /// Arguments passed to the program
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        program_args: Vec<String>,
    },

    /// Check a source file for errors without producing output files
    Check {
        /// Source path
        #[arg(value_parser = existing_file_path)]
        path: PathBuf,

        /// Cache directory
        #[arg(long)]
        cache: Option<String>,

        /// Reset the build cache before checking
        #[arg(long)]
        update: bool,
    },


    /// Compile and run tests
    Test {
        /// Source path
        #[arg(default_value = ".", value_parser = existing_path)]
        path: PathBuf,

        /// Test name filter
        filter: Option<String>,

        /// Cache directory
        #[arg(long)]
        cache: Option<String>,

        /// Reset the build cache before testing
        #[arg(long)]
        update: bool,
    },

    /// Remove build artifacts
    Clean {
        /// Cache directory
        #[arg(long)]
        cache: Option<String>,
    },

}


#[derive(Subcommand)]
enum LibCommands {
    /// Initialize a library project
    Init {
        /// Project path
        path: Option<String>,
    },

    /// Build the library
    Build,
}


/// Process exit codes, complementing clap's own usage-error code (2).
mod exit {
    /// Compilation reported errors.
    pub const COMPILE_ERROR: i32 = 1;
    /// A link or other external toolchain step failed.
    pub const LINK_ERROR: i32 = 3;
    /// The program launched by `run` exited non-zero; the child's own code is
    /// propagated when it fits the portable 0..=125 range.
    pub const PROGRAM_ERROR: i32 = 4;
}

/// Prints a clap-styled error and terminates with `code`.
fn fail(code: i32, message: impl std::fmt::Display) -> ! {
    use clap::error::{Error, ErrorKind};

    let mut cmd = <Cli as clap::CommandFactory>::command();
    let error: Error = Error::raw(ErrorKind::Io, message).format(&mut cmd);

    // `Error::exit` hardcodes clap's usage code (2); render the styled
    // message ourselves and exit with the specific status.
    let _ = error.print();
    std::process::exit(code);
}

fn main() {
    let Cli { command } = Cli::parse();

    // `lib` manages its own project directory, not the shared artifacts cache.
    let _lock =
        if matches!(command, Commands::Lib { .. }) { None }
        else { Some(ArtifactsLock::acquire()) };

    match command {
        Commands::Lib { command } => match command {
            LibCommands::Init { path } => {
                let cd = std::env::current_dir().unwrap();
                let path =
                    path.map(|s| cd.join(s))
                        .unwrap_or(cd);

                margarine::library::validate_prerequisites().unwrap();
                margarine::library::init(path).unwrap();
            }

            LibCommands::Build => {
                let path = std::env::current_dir().unwrap();
                if let Err(error) = margarine::library::build(&path) {
                    fail(exit::LINK_ERROR, format!("cannot build library: {error}"));
                }
            }
        }

        Commands::Build { path, target, output: _, cache, update } => {
            let cache = reset_cache_if(update, cache);
            compile_and_link(&path, target, None, Some(cache));
        }
        Commands::Run { path, cache, update, program_args } => {
            let cache = reset_cache_if(update, cache);
            let output =
            compile_and_link(&path, CompilationTarget::try_from("default").unwrap(), None, Some(cache));

            println!("running '{output}'");
            let status = Command::new(&output)
                .args(program_args)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()
                .unwrap();
            if !status.success() {
                let code = status.code().unwrap_or(exit::PROGRAM_ERROR);
                std::process::exit(code.clamp(0, 125));
            }
        }

        Commands::Test { path, filter, cache, update } => {
            let cache = reset_cache_if(update, cache);
            let program = format!("{cache}/program");
            let arena = Arena::new();
            let mut compiler = margarine::Compiler::new(&arena);
            let file = margarine::FileData::open(
                &path.display().to_string(),
                &mut compiler.string_map,
            ).unwrap();
            let entry = compiler.string_map.get(file.name()).into();
            compiler.files.register(file);

            let settings = CompilationSettings {
                compilation_target: CompilationTarget::try_from("default").unwrap(),
                preludes: parse_env_preludes(),
                entry,
                output: program.clone(),
                cache: cache.clone(),
                arena: &arena,
                tests: true,
            };

            let mut result = compiler.run(&settings);
            let errors = compiler.check(&mut result);
            compiler.codegen(&settings, &mut result, errors);
            let link_files = result.link_files().to_vec();

            let tests = result.tests().iter()
                .map(|(sym, should_panic)| (
                    compiler.string_map.get(result.syms.sym(*sym).name()).to_string(),
                    *should_panic,
                ))
                .collect::<Vec<_>>();

            let mut clang = Command::new("clang");
            clang.arg("-shared")
                .arg(format!("{program}.o"))
                .args(&link_files)
                .arg("-lzstd")
                .arg("-lz")
                .arg("-lc++")
                .arg("-lc++abi")
                .arg("-o")
                .arg(format!("{program}.dylib"));
            if !run_step("linking...", &mut clang) {
                fail(exit::LINK_ERROR, "linking failed");
            }

            let success = run_tests(&tests, filter, &format!("{program}.dylib"));
            std::process::exit(!success as i32);
        }

        Commands::Check { path, cache, update } => {
            let cache = reset_cache_if(update, cache);
            let arena = Arena::new();
            let mut compiler = margarine::Compiler::new(&arena);
            let file = margarine::FileData::open(
                &path.display().to_string(),
                &mut compiler.string_map,
            ).unwrap();
            let entry = compiler.string_map.get(file.name()).into();
            compiler.files.register(file);

            let settings = CompilationSettings {
                compilation_target: CompilationTarget::try_from("default").unwrap(),
                preludes: parse_env_preludes(),
                entry,
                output: String::new(),
                cache,
                arena: &arena,
                tests: false,
            };

            let mut result = compiler.run(&settings);
            let errors = compiler.check(&mut result);
            let error_count = errors.iter().flatten().map(|file| file.len()).sum::<usize>();

            if error_count == 0 {
                println!("{}", "no errors found".green());
            } else {
                std::process::exit(exit::COMPILE_ERROR);
            }
        }

        Commands::Clean { cache } => {
            clean_artifacts(&cache.unwrap_or_else(|| "artifacts".to_string()));
        }
    }
}


/// Compiles `path` with the Compiler + CompilationResult pipeline and links
/// the result for `target`. Shared by `build` and `run`.
fn compile_and_link(
    path: &PathBuf,
    target: CompilationTarget,
    output: Option<String>,
    cache: Option<String>,
) -> String {
    let cache = cache.unwrap_or("artifacts".to_string());
    let output = output
        .map(|s| PathBuf::from(s))
        .unwrap_or({
            let path = path.with_extension("");
            let name = path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or("program".into());
            let path = PathBuf::from(&cache);
            let path = path.join(name);
            let path = path.with_extension(target.output_suffix());
            path
        });

    let output = output.to_string_lossy();

    let arena = Arena::new();
    let mut compiler = margarine::Compiler::new(&arena);
    let file = margarine::FileData::open(
        &path.display().to_string(),
        &mut compiler.string_map,
    ).unwrap();
    let entry = compiler.string_map.get(file.name()).into();
    compiler.files.register(file);

    let settings = CompilationSettings {
        compilation_target: target,
        preludes: parse_env_preludes(),
        entry,
        output: output.to_string(),
        cache: cache.to_string(),
        arena: &arena,
        tests: false,
    };

    let mut result = compiler.run(&settings);
    let errors = compiler.check(&mut result);
    let error_count = errors.iter().flatten().map(|file| file.len()).sum::<usize>();

    // Codegen still runs unconditionally after diagnostics (repo policy), but
    // a compile that produced errors must not report success.
    if error_count > 0 {
        fail(exit::COMPILE_ERROR,
            format!("compilation failed with {error_count} error(s)"));
    }

    compiler.codegen(&settings, &mut result, errors);
    let link_files = result.link_files().to_vec();

    let link_ok = match target {
        CompilationTarget::Arm64AppleDarwin => {
            let mut clang = Command::new("clang");
            clang.arg(format!("{output}.o"))
                .args(&link_files)
                .arg("-lzstd")
                .arg("-lz")
                .arg("-lc++")
                .arg("-lc++abi")
                .arg("-o")
                .arg(&*output);
            run_step("linking...", &mut clang)
        }
        CompilationTarget::Wasm32UnknownUnknown => {
            let mut linker = Command::new("wasm-ld");
            linker.arg("--no-entry")
                .arg("--export=main")
                .arg("--export-memory")
                .arg(format!("{output}.o"))
                .args(&link_files)
                .arg("-o")
                .arg(&*output);
            run_step("linking browser wasm...", &mut linker)
        }
    };

    if !link_ok {
        fail(exit::LINK_ERROR, "linking failed");
    }

    output.into()
}


/// Exclusive cross-process lock over `artifacts/`, held for the lifetime of
/// the returned guard. The lock file lives outside the deleted tree
/// (`artifacts.lock` next to `build.lock`) because removing a lock file while
/// held would leave latecomers locking a fresh inode with no mutual exclusion.
struct ArtifactsLock {
    file: std::fs::File,
}

impl ArtifactsLock {
    fn acquire() -> Self {
        use fs2::FileExt;

        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open("artifacts.lock")
            .unwrap_or_else(|error| panic!("cannot open artifacts.lock: {error}"));

        // Try non-blocking first so the uncontended case stays silent.
        if file.try_lock_exclusive().is_err() {
            eprintln!("{}", "waiting for artifacts lock...".dim());
            file.lock_exclusive()
                .unwrap_or_else(|error| panic!("cannot lock artifacts: {error}"));
        }

        ArtifactsLock { file }
    }
}

impl Drop for ArtifactsLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

fn clean_artifacts(cache: &str) {
    if !std::fs::exists(cache).unwrap_or(false) {
        println!("{}", "nothing to clean".dim());
        return;
    }

    let mut paths: Vec<PathBuf> = Vec::new();
    let mut stack = vec![PathBuf::from(cache)];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                paths.push(path);
            }
        }
    }

    let total_bytes: u64 = paths.iter()
        .filter_map(|path| std::fs::metadata(path).ok())
        .map(|meta| meta.len())
        .sum();

    let total = format_bytes(total_bytes);
    let bar_width = 30;
    let tick = |done: usize| -> String {
        let filled = done * bar_width / paths.len().max(1);
        format!(
            "[{}{}] {}/{}",
            "=".repeat(filled),
            " ".repeat(bar_width - filled),
            done,
            paths.len(),
        )
    };

    let is_tty = unsafe { libc::isatty(libc::STDOUT_FILENO) } == 1;
    let progress = |done: usize| {
        if !is_tty { return; }
        print!("\r     Removing {total} {}",
            tick(done),
        );
        let _ = io::stdout().flush();
    };

    progress(0);
    let mut failures = 0usize;
    for (index, path) in paths.iter().enumerate() {
        if std::fs::remove_file(path).is_err() {
            failures += 1;
            eprintln!("{} could not remove {}", "warning:".yellow().bold(), path.display());
        }
        progress(index + 1);
    }

    if let Err(error) = std::fs::remove_dir_all(cache) {
        eprintln!("{} could not remove {cache}: {error}", "warning:".yellow().bold());
        failures += 1;
    }

    if is_tty {
        print!("\r");
    }
    if failures == 0 {
        println!("{} Removed {} files, {}",
            "✓".green(),
            paths.len(),
            total.cyan(),
        );
    } else {
        println!("{} removed {} files, {} failed; use `--update` to reset the cache",
            "!".yellow().bold(),
            paths.len() - failures,
            failures,
        );
    }
}


/// Resolves the effective cache directory, resetting the build cache first
/// when `update` is set. The lock file lives at the project root and is
/// cache-independent; the commit map it holds is only consulted against the
/// fresh clones a reset triggers anyway.
fn reset_cache_if(update: bool, cache: Option<String>) -> String {
    let cache = cache.unwrap_or_else(|| "artifacts".to_string());
    if update {
        if std::fs::exists("build.lock").unwrap() {
            std::fs::remove_file("build.lock").unwrap();
        }

        clean_artifacts(&cache);
    }

    cache
}

fn format_bytes(bytes: u64) -> String {
    let units = ["b", "KiB", "MiB", "GiB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < units.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes}b")
    } else {
        format!("{value:.1}{}", units[unit])
    }
}

fn existing_path(path: &str) -> Result<PathBuf, String> {
    if std::fs::exists(path).unwrap_or(false) {
        Ok(PathBuf::from(path))
    } else {
        Err(format!("path does not exist: {path}"))
    }
}

fn existing_file_path(path: &str) -> Result<PathBuf, String> {
    let path = existing_path(path)?;
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("path is not a file: {}", path.display()))
    }
}


fn run_step(label: &str, cmd: &mut Command) -> bool {
    println!("{}", label.green().bold());
    match cmd.output() {
        Ok(output) if output.status.success() => true,
        Ok(output) => {
            eprintln!("{} failed with {}", label, output.status);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stdout.trim().is_empty() {
                eprintln!("{stdout}");
            }
            if !stderr.trim().is_empty() {
                eprintln!("{stderr}");
            }
            false
        }
        Err(err) => {
            eprintln!("{} failed to start: {err}", label);
            false
        }
    }
}


fn run_tests(tests: &[(String, bool)], filter: Option<String>, dylib: &str) -> bool {
    if tests.is_empty() {
        println!();
        println!("running 0 tests");
        println!();
        println!("test result: ok. 0 passed; 0 failed; 0 ignored; finished in 0.00s");
        return true;
    }

    let start = Instant::now();

    let timeout_ms: u64 = std::env::var("MARGARINE_TEST_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);

    let filter = filter.or_else(|| std::env::var("MARGARINE_TEST_FILTER").ok());

    unsafe {
        let lib_path = CString::new(dylib).unwrap();
        let lib = libc::dlopen(lib_path.as_ptr(), libc::RTLD_NOW);
        if lib.is_null() {
            println!("failed to load {dylib}");
            return false;
        }

        println!();
        println!("running {} tests", tests.len());
        println!();

        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut ignored = 0u32;
        let mut fails = String::new();

        for (name, should_panic) in tests {
            if let Some(ref filter) = filter {
                if !name.contains(filter.as_str()) {
                    ignored += 1;
                    continue;
                }
            }

            let label = if *should_panic { " - should panic" } else { "" };
            print!("test '{}'{} ... ", name, label);
            io::stdout().flush().unwrap();

            let func = lookup_test(lib, name);
            if func.is_null() {
                println!("{}", "FAILED".red());
                failed += 1;
                writeln!(&mut fails, "failed '{}': function not found in dylib", name).unwrap();
                continue;
            }

            let func: unsafe extern "C" fn(*const u8) = std::mem::transmute(func);

            let mut pipe_fds: [i32; 2] = [0; 2];
            libc::pipe(pipe_fds.as_mut_ptr());

            let pid = libc::fork();
            if pid == 0 {
                libc::close(pipe_fds[0]);
                libc::dup2(pipe_fds[1], 1);
                libc::dup2(pipe_fds[1], 2);
                libc::close(pipe_fds[1]);
                func(std::ptr::null());
                libc::exit(0);
            }

            libc::close(pipe_fds[1]);

            let mut status: i32 = 0;
            let mut timed_out = false;
            let poll_start = Instant::now();

            loop {
                let ret = libc::waitpid(pid, &mut status, libc::WNOHANG);
                if ret != 0 { break }

                if poll_start.elapsed().as_millis() as u64 >= timeout_ms {
                    libc::kill(pid, libc::SIGKILL);
                    libc::waitpid(pid, &mut status, 0);
                    timed_out = true;
                    break;
                }

                libc::usleep(10);
            }

            let output = read_pipe(pipe_fds[0]);
            libc::close(pipe_fds[0]);

            let exited_ok = wifexited(status) && wexitstatus(status) == 0;

            if timed_out {
                println!("{}", "FAILED".red());
                failed += 1;
                writeln!(&mut fails, "failed '{}' (timed out after {}ms):\n{}",
                    name,
                    timeout_ms,
                    output.trim(),
                ).unwrap();
            } else if *should_panic {
                if !exited_ok {
                    println!("{}", "ok".green());
                    passed += 1;
                } else {
                    println!("{}", "FAILED".red());
                    failed += 1;
                    writeln!(&mut fails, "failed '{}' (exit code 0): test did not panic as expected", name).unwrap();
                }
            } else {
                if exited_ok {
                    println!("{}", "ok".green());
                    passed += 1;
                } else {
                    println!("{}", "FAILED".red());
                    failed += 1;
                    let reason = if wifsignaled(status) {
                        format!(" (signal {})", wtermsig(status))
                    } else if wifexited(status) {
                        format!(" (exit code {})", wexitstatus(status))
                    } else {
                        String::new()
                    };
                    writeln!(&mut fails, "failed '{}'{}:\n{}",
                        name,
                        reason,
                        output.trim(),
                    ).unwrap();
                }
            }
        }

        libc::dlclose(lib);

        println!();
        if !fails.is_empty() {
            println!("failures:");
            println!();
            println!("{}", fails);
            println!();
        }

        let elapsed = start.elapsed();
        let result = if failed == 0 { "ok".green() } else { "FAILED".red() };
        println!(
            "test result: {}. {} passed; {} failed; {} ignored; finished in {:.2}s",
            result, passed, failed, ignored, elapsed.as_secs_f64()
        );
        println!();
        failed == 0
    }
}


unsafe fn lookup_test(lib: *mut libc::c_void, name: &str) -> *mut libc::c_void {
    let cname = CString::new(name).unwrap();
    let ptr = libc::dlsym(lib, cname.as_ptr());
    if !ptr.is_null() {
        return ptr;
    }

    let cname = CString::new(format!("_{name}")).unwrap();
    let ptr = libc::dlsym(lib, cname.as_ptr());
    if !ptr.is_null() {
        return ptr;
    }

    std::ptr::null_mut()
}


unsafe fn read_pipe(fd: i32) -> String {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    loop {
        let n = libc::read(fd, tmp.as_mut_ptr().cast(), tmp.len());
        if n <= 0 { break; }
        buf.extend_from_slice(&tmp[..n as usize]);
    }
    String::from_utf8_lossy(&buf).into_owned()
}


fn wifexited(status: i32) -> bool {
    status & 0x7f == 0
}


fn wexitstatus(status: i32) -> i32 {
    (status >> 8) & 0xff
}


fn wifsignaled(status: i32) -> bool {
    ((status & 0x7f) + 1) >> 1 > 0
}


fn wtermsig(status: i32) -> i32 {
    status & 0x7f
}


fn parse_env_preludes() -> Vec<Prelude> {
    let preludes = 
    std::env::var("MARGARINE_PRELUDE")
        .iter()
        .flat_map(|s| s.split(';'))
        .filter_map(|s| s.split_once('='))
        .map(|(alias, url)| Prelude { alias: alias.into(), url: url.into() })
        .collect::<Vec<_>>();


    if preludes.is_empty() {
        let url = format!(
            "https://cdn.daymare.net/margarine/{}/share", 
            env!("CARGO_PKG_VERSION")
        );

        vec![
            Prelude { alias: "core".into(), url: format!("{url}/core") },
            Prelude { alias: "std".into(), url: format!("{url}/std") },
        ]
    } else {
        preludes
    }
}
