use std::{ffi::CString, fmt::Write, io::{self, Write as _}, process::Command, time::Instant};

use colourful::ColourBrush;
use common::{source::FileData, string_map::StringMap};
use sti::{arena::Arena};

fn main() {
    let mut args = std::env::args().skip(1);

    let Some(command) = args.next()
    else { 
        println!("invalid command");
        return;
    };

    match command.as_str() {
        "run" => {
            let path = args.next().unwrap();
            let arena = Arena::new();
            let mut sm = StringMap::new(&arena);
            let files = FileData::open(path, &mut sm).unwrap();
            let (_, _) = margarine::run(sm, files, false);

            println!("{:?}",
                Command::new("llc")
                    .arg("-filetype=obj")
                    .arg("artifacts/out.ll")
                    .arg("-o=artifacts/program.o")
                    .output()
            );

            println!("{:?}",
                Command::new("clang")
                    .arg("artifacts/program.o")
                    .arg("libmargarine.a")
                    .arg("-lzstd")
                    .arg("-lz")
                    .arg("-lc++")
                    .arg("-lc++abi")
                    .arg("-o")
                    .arg("artifacts/program")
                    .output()
            );

            println!("{}",
                std::str::from_utf8(&Command::new("./artifacts/program")
                    .output()
                    .unwrap()
                    .stdout
                ).unwrap()
            );
            return;
        },


        "test" => {
            let path = args.next().unwrap_or_else(|| ".".to_string());
            let arena = Arena::new();
            let mut sm = StringMap::new(&arena);
            let files = FileData::open(path, &mut sm).unwrap();
            let (_, tests) = margarine::run(sm, files, true);

            println!("{:?}",
                Command::new("llc")
                    .arg("-O2")
                    .arg("-filetype=obj")
                    .arg("-relocation-model=pic")
                    .arg("artifacts/out.ll")
                    .arg("-o=artifacts/program.o")
                    .output()
            );

            println!("{:?}",
                Command::new("clang")
                    .arg("-shared")
                    .arg("libmargarine.a")
                    .arg("artifacts/program.o")
                    .arg("-lzstd")
                    .arg("-lz")
                    .arg("-lc++")
                    .arg("-lc++abi")
                    .arg("-o")
                    .arg("artifacts/program.dylib")
                    .output()
            );

            run_tests(&tests);
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


fn run_tests(tests: &[(String, bool)]) {
    if tests.is_empty() {
        println!();
        println!("running 0 tests");
        println!();
        println!("test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s");
        return;
    }

    let start = Instant::now();

    let timeout_ms: u64 = std::env::var("MARGARINE_TEST_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);

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
        let mut fails = String::new();

        for (name, should_panic) in tests {
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
            "test result: {}. {} passed; {} failed; 0 ignored; 0 measured; 0 filtered out; finished in {:.2}s",
            result, passed, failed, elapsed.as_secs_f64()
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
