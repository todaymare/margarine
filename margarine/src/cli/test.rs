use std::{ffi::CString, fmt::Write, path::Path, process::Command, time::Instant};

use colourful::ColourBrush;
use margarine::{progress::{item_progress, StatusLine}, resource, start_compilation_status, CompilationSettings, CompilationTarget};
use sti::arena::Arena;

use super::{CliError, CliResult, COMPILE_ERROR, X_GLYPH};

pub(super) fn execute(
    path: &Path,
    filter: Option<String>,
    target: CompilationTarget,
    cache: String,
) -> CliResult<i32> {
    let program = format!("{cache}/program");
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
        output: program.clone(),
        cache: cache.into(),
        arena: &arena,
        tests: true,
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
    compiler.codegen(&settings, &mut result, errors);
    let mut link_files = result.link_files().to_vec();
    if let Err(error) = margarine::prepare_link_files(target, &mut link_files) {
        compile_status.clear();
        return Err(CliError::link(error));
    }

    let tests = result.tests().iter()
        .map(|(sym, should_panic)| (
            compiler.string_map.get(result.syms.sym(*sym).name()).to_string(),
            *should_panic,
        ))
        .collect::<Vec<_>>();
    let dylib = format!("{program}.{}", target.shared_library_suffix());
    let toolchain_libs = resource::toolchain_libs_path(target);

    let link_ok = match target {
        CompilationTarget::Arm64AppleDarwin => {
            let mut clang = Command::new("clang");
            clang.arg("-target")
                .arg(target.c_target_triple())
                .arg("-shared")
                .arg("-L")
                .arg(&toolchain_libs)
                .arg(format!("{program}.o"))
                .args(&link_files)
                .arg("-lz")
                .arg("-lc++")
                .arg("-lc++abi")
                .arg("-o")
                .arg(&dylib);
            run_step("Linking test library", &mut clang)
        },
        CompilationTarget::X86_64UnknownLinuxGnu
        | CompilationTarget::Aarch64UnknownLinuxGnu => {
            let mut clang = Command::new("clang");
            clang.arg("-target")
                .arg(target.c_target_triple())
                .arg("-shared")
                .arg("-L")
                .arg(&toolchain_libs)
                .arg(format!("{program}.o"))
                .args(&link_files)
                .arg("-lz")
                .arg("-lstdc++")
                .arg("-o")
                .arg(&dylib);
            run_step("Linking test library", &mut clang)
        },
        CompilationTarget::Wasm32UnknownUnknown => {
            compile_status.clear();
            return Err(CliError::link("tests do not support the wasm32-unknown-unknown target"));
        },
    };
    if !link_ok {
        compile_status.clear();
        return Err(CliError::link("linking failed"));
    }
    compile_status.finish(format!("Built {}", path.display()));

    Ok(if run_tests(&tests, filter, &dylib) { 0 } else { COMPILE_ERROR })
}

fn run_step(label: &str, cmd: &mut Command) -> bool {
    let status = StatusLine::start(label);
    match cmd.output() {
        Ok(output) if output.status.success() => {
            status.finish("Linked test library");
            true
        }
        Ok(output) => {
            status.suspend(|| {
                eprintln!("{} {} failed with {}", X_GLYPH.red().bold(), label, output.status);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stdout.trim().is_empty() {
                    eprintln!("{stdout}");
                }
                if !stderr.trim().is_empty() {
                    eprintln!("{stderr}");
                }
            });
            status.clear();
            false
        }
        Err(err) => {
            status.suspend(
                || eprintln!("{} {} failed to start: {err}", X_GLYPH.red().bold(), label),
            );
            status.clear();
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
        let progress = item_progress(tests.len() as u64, "Testing");


        for (name, should_panic) in tests {
            if let Some(ref filter) = filter {
                if !name.contains(filter.as_str()) {
                    ignored += 1;
                    progress.inc(1);
                    continue;
                }
            }

            let label = if *should_panic { " - should panic" } else { "" };
            progress.set_message(format!("Testing {name}"));

            let func = lookup_test(lib, name);
            if func.is_null() {
                progress.suspend(|| {
                    println!("test '{}'{} ... {}", name, label, "FAILED".red());
                });
                failed += 1;
                writeln!(&mut fails, "failed '{}': function not found in dylib", name).unwrap();
                progress.inc(1);
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
                progress.suspend(|| {
                    println!("test '{}'{} ... {}", name, label, "FAILED".red());
                });
                failed += 1;
                writeln!(&mut fails, "failed '{}' (timed out after {}ms):\n{}",
                    name,
                    timeout_ms,
                    output.trim(),
                ).unwrap();
            } else if *should_panic {
                if !exited_ok {
                    progress.suspend(|| {
                        println!("test '{}'{} ... {}", name, label, "ok".green());
                    });
                    passed += 1;
                } else {
                    progress.suspend(|| {
                        println!("test '{}'{} ... {}", name, label, "FAILED".red());
                    });
                    failed += 1;
                    writeln!(&mut fails, "failed '{}' (exit code 0): test did not panic as expected", name).unwrap();
                }
            } else {
                if exited_ok {
                    progress.suspend(|| {
                        println!("test '{}'{} ... {}", name, label, "ok".green());
                    });
                    passed += 1;
                } else {
                    progress.suspend(|| {
                        println!("test '{}'{} ... {}", name, label, "FAILED".red());
                    });
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
            progress.inc(1);
        }

        libc::dlclose(lib);
        progress.finish();

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
