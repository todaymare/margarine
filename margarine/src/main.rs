use std::{ffi::CString, fmt::Write, io::{self, Write as _}, process::Command, time::Instant};

use colourful::ColourBrush;
use margarine::{CompilationSettings, CompilationTarget, Prelude};
use sti::{arena::Arena};

fn main() {
    let mut args = std::env::args().skip(1);

    let Some(command) = args.next()
    else { 
        println!("invalid command");
        return;
    };

    if matches!(command.as_str(), "--version" | "-V") {
        println!("margarine {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    match command.as_str() {
        "lib" => {
            let cmd = args.next();
            match cmd.unwrap_or("".into()).as_str() {
                "init" => {
                    let cd = std::env::current_dir().unwrap();
                    let path = args.next()
                        .map(|s| cd.join(s))
                        .unwrap_or(cd);

                    margarine::library::validate_prerequisites().unwrap();
                    margarine::library::init(path).unwrap();
                }


                "build" => {
                    let path = std::env::current_dir().unwrap();
                    if let Err(error) = margarine::library::build(&path) {
                        eprintln!("cannot build library: {error}");
                        return;
                    }
                }

                _ => {
                    println!("help menu");
                }
            }
        }


        "build" => {
            let mut target = CompilationTarget::Arm64AppleDarwin;
            let mut path = None;
            let mut output = None;
            let mut cache = None;

            while let Some(arg) = args.next() {
                if arg == "--target" {
                    let Some(value) = args.next() else {
                        eprintln!("missing value for --target");
                        return;
                    };
                    target = match CompilationTarget::try_from(value.as_str()) {
                        Ok(target) => target,
                        Err(error) => {
                            eprintln!("{error}");
                            return;
                        }
                    };
                } else if arg == "-o" {
                    let Some(value) = args.next() else {
                        eprintln!("missing value for -o");
                        return;
                    };
                    if output.replace(value).is_some() {
                        eprintln!("build accepts exactly one output path");
                        return;
                    }
                } else if arg == "--cache" {
                    let Some(value) = args.next() else {
                        eprintln!("missing value for --cache");
                        return;
                    };
                    if cache.replace(value).is_some() {
                        eprintln!("build accepts exactly one cache path");
                        return;
                    }
                } else if path.replace(arg).is_some() {
                    eprintln!("build accepts exactly one source path");
                    return;
                }
            }

            let Some(path) = path
            else {
                eprintln!("missing source path");
                return;
            };

            let output = output.unwrap_or({
                match target {
                    CompilationTarget::Arm64AppleDarwin => "artifacts/program".into(),
                    CompilationTarget::Wasm32UnknownUnknown => "artifacts/program.wasm".into(),
                }
            });

            let cache = cache.unwrap_or("artifacts".to_string());

            let arena = Arena::new();
            let (link_files, _) = margarine::run(CompilationSettings {
                compilation_target: target,
                preludes: parse_env_preludes(),
                entry: path,
                output: output.clone(),
                cache,
                arena: &arena,
                tests: false,
            });

            match target {
                CompilationTarget::Arm64AppleDarwin => {
                    let mut clang = Command::new("clang");
                    clang.arg(format!("{output}.o"))
                        .args(&link_files)
                        .arg("-lzstd")
                        .arg("-lz")
                        .arg("-lc++")
                        .arg("-lc++abi")
                        .arg("-o")
                        .arg(output);
                    run_step("linking...", &mut clang);
                }
                CompilationTarget::Wasm32UnknownUnknown => {
                    let mut linker = Command::new("wasm-ld");
                    linker.arg("--no-entry")
                        .arg("--export=main")
                        .arg("--export-memory")
                        .arg(format!("{output}.o"))
                        .args(&link_files)
                        .arg("-o")
                        .arg(output);
                    run_step("linking browser wasm...", &mut linker);
                }
            }
            return;
        },

        "run" => {
            let path = args.next().unwrap();
            let program_args = args.collect::<Vec<_>>();
            let arena = Arena::new();
            let (link_files, _) = margarine::run(CompilationSettings {
                compilation_target: CompilationTarget::try_from("default").unwrap(),
                preludes: parse_env_preludes(),
                entry: path,
                output: "artifacts/program".to_string(),
                cache: "artifacts".into(),
                arena: &arena,
                tests: false,
            });

            let mut clang = Command::new("clang");
            clang.arg("artifacts/program.o")
                .args(&link_files)
                .arg("-lzstd")
                .arg("-lz")
                .arg("-lc++")
                .arg("-lc++abi")
                .arg("-o")
                .arg("artifacts/program");
            if !run_step("linking...", &mut clang) {
                return;
            }

            println!("running");
            let status = Command::new("./artifacts/program")
                .args(program_args)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()
                .unwrap();
            if !status.success() {
                eprintln!("program exited with {}", status);
            }
            return;
        },


        "test" => {
            let path = args.next().unwrap_or_else(|| ".".to_string());
            let filter = args.next();
            let arena = Arena::new();
            let (link_files, tests) = margarine::run(CompilationSettings {
                compilation_target: CompilationTarget::try_from("default").unwrap(),
                preludes: parse_env_preludes(),
                entry: path,
                output: "artifacts/program".to_string(),
                cache: "artifacts".into(),
                arena: &arena,
                tests: true,
            });

            let mut clang = Command::new("clang");
            clang.arg("-shared")
                .arg("artifacts/program.o")
                .args(&link_files)
                .arg("-lzstd")
                .arg("-lz")
                .arg("-lc++")
                .arg("-lc++abi")
                .arg("-o")
                .arg("artifacts/program.dylib");

            if !run_step("linking...", &mut clang) {
                return;
            }

            run_tests(&tests, filter);
            return;
        },


        "clean" => {
            if std::fs::exists("artifacts").unwrap() {
                std::fs::remove_dir_all("artifacts").unwrap();
            }
        }


        "update" => {
            if std::fs::exists("build.lock").unwrap() {
                std::fs::remove_file("build.lock").unwrap();
            }

            if std::fs::exists("artifacts").unwrap() {
                std::fs::remove_dir_all("artifacts").unwrap();
            }
        }


        _ => {
            println!("invalid command");
            return;
        }
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


fn run_tests(tests: &[(String, bool)], filter: Option<String>) {
    if tests.is_empty() {
        println!();
        println!("running 0 tests");
        println!();
        println!("test result: ok. 0 passed; 0 failed; 0 ignored; finished in 0.00s");
        return;
    }

    let start = Instant::now();

    let timeout_ms: u64 = std::env::var("MARGARINE_TEST_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);

    let filter = filter.or_else(|| std::env::var("MARGARINE_TEST_FILTER").ok());

    unsafe {
        let lib_path = CString::new("artifacts/program.dylib").unwrap();
        let lib = libc::dlopen(lib_path.as_ptr(), libc::RTLD_NOW);
        if lib.is_null() {
            println!("failed to load artifacts/program.dylib");
            return;
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
