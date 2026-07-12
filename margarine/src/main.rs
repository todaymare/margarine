use std::{ffi::CString, fmt::Write, io::{self, Write as _}, time::Instant};

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
            let (_, _) = margarine::run(sm, files);

            println!("running");
            return;
        },


        "test" => {
            let path = args.next().unwrap_or_else(|| ".".to_string());
            let arena = Arena::new();
            let mut sm = StringMap::new(&arena);
            let files = FileData::open(path, &mut sm).unwrap();
            let (_, tests) = margarine::test(sm, files);

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

    unsafe {
        let lib_path = CString::new("program.dylib").unwrap();
        let lib = libc::dlopen(lib_path.as_ptr(), libc::RTLD_NOW);
        if lib.is_null() {
            println!("failed to load program.dylib");
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
            libc::waitpid(pid, &mut status, 0);

            let output = read_pipe(pipe_fds[0]);
            libc::close(pipe_fds[0]);

            let exited_ok = wifexited(status) && wexitstatus(status) == 0;

            if *should_panic {
                if !exited_ok {
                    println!("{}", "ok".green());
                    passed += 1;
                } else {
                    println!("{}", "FAILED".red());
                    failed += 1;
                    writeln!(&mut fails, "failed '{}': test did not panic as expected", name).unwrap();
                }
            } else {
                if exited_ok {
                    println!("{}", "ok".green());
                    passed += 1;
                } else {
                    println!("{}", "FAILED".red());
                    failed += 1;
                    writeln!(&mut fails, "failed '{}':\n{}",
                        name,
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
