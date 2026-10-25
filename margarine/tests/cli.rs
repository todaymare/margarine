use std::{
    fs::{self, File, OpenOptions},
    os::fd::FromRawFd as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use fs2::FileExt;
use margarine::{
    resource, Arena, CompilationSettings, CompilationTarget, Compiler, FileData, VERSION,
};


fn prelude_fixture(root: &Path) -> PathBuf {
    let prelude = root.join("prelude");
    fs::create_dir_all(prelude.join("lib")).unwrap();
    fs::write(prelude.join("lib.mar"), "pub mod prelude;").unwrap();
    fs::write(prelude.join("lib/prelude.mar"), "").unwrap();

    let repository = git2::Repository::init(&prelude).unwrap();
    let mut index = repository.index().unwrap();
    index.add_path(Path::new("lib.mar")).unwrap();
    index.add_path(Path::new("lib/prelude.mar")).unwrap();
    let tree = repository.find_tree(index.write_tree().unwrap()).unwrap();
    let signature = git2::Signature::now("margarine", "margarine@localhost").unwrap();
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

    prelude
}


#[test]
fn runtime_errors_end_with_a_newline() {
    let output = Command::new(env!("CARGO_BIN_EXE_margarine"))
        .arg("update")
        .env("MARGARINE_RELEASES_API", "http://127.0.0.1:0")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.starts_with(b"error:"));
    assert_eq!(output.stderr.last(), Some(&b'\n'));
    assert!(!output.stderr.ends_with(b"\n\n"));
}


#[test]
fn bare_unmanaged_invocation_prints_help_without_installing() {
    let home = tempfile::tempdir().unwrap();
    let mut master_fd = -1;
    let mut slave_fd = -1;
    assert_eq!(
        unsafe {
            libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        0,
    );
    let _master =
        unsafe { File::from_raw_fd(master_fd) };
    let slave =
        unsafe { File::from_raw_fd(slave_fd) };
    let stdout = slave.try_clone().unwrap();
    let stderr = slave.try_clone().unwrap();

    let status =
        Command::new(env!("CARGO_BIN_EXE_margarine"))
            .env("HOME", home.path())
            .env("MARGARINE_RELEASES_API", "http://127.0.0.1:1/unreachable")
            .stdin(Stdio::from(slave))
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .status()
            .unwrap();

    assert_eq!(status.code(), Some(2));

    assert!(!home.path().join(".margarine").exists());
}



#[test]
fn toolchain_add_rejects_unsupported_target() {
    let output = Command::new(env!("CARGO_BIN_EXE_margarine"))
        .args(["toolchain", "add", "not-a-target"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.starts_with(b"error:"));
}




#[test]
fn piped_lock_wait_is_transient() {
    let dir = tempfile::tempdir().unwrap();
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.path().join("artifacts.lock"))
        .unwrap();
    lock.lock_exclusive().unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_margarine"))
        .arg("clean")
        .current_dir(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_millis(150));
    assert!(child.try_wait().unwrap().is_none());

    lock.unlock().unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"nothing to clean\n");
    assert!(output.stderr.is_empty());
}


#[test]
fn run_displays_the_source_path_instead_of_the_executable_path() {
    let dir = tempfile::tempdir().unwrap();
    let prelude = prelude_fixture(dir.path());

    fs::write(dir.path().join("program.mar"), "fn main() {}").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_margarine"))
        .args(["run", "--cache", "artifacts", "program.mar"])
        .current_dir(dir.path())
        .env("MARGARINE_PRELUDE", format!("fixture={}", prelude.display()))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(output.stdout, "› Running program.mar\n".as_bytes());
}


#[test]
fn run_forwards_native_process_arguments() {
    let dir = tempfile::tempdir().unwrap();
    let prelude = prelude_fixture(dir.path());
    fs::write(
        dir.path().join("program.mar"),
        r#"extern {
    fn margarineEnvArgs(): [str]
}

fn main(): int {
    let args = margarineEnvArgs();
    let second_argument = args[2];
    0
}
"#,
    ).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_margarine"))
        .args([
            "run",
            "--cache",
            "artifacts",
            "program.mar",
            "first",
            "second",
        ])
        .current_dir(dir.path())
        .env("MARGARINE_PRELUDE", format!("fixture={}", prelude.display()))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn c_abi_margarine_and_c_driver_interoperate() {
    let workspace = tempfile::tempdir().unwrap();
    let prelude = prelude_fixture(workspace.path());
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/native");
    let source = fixtures.join("c_abi_interop.mar");
    let helper = fixtures.join("c_abi_helpers.c");
    let driver = fixtures.join("c_abi_driver.c");
    let dylib = workspace.path().join(format!(
        "c_abi_interop.{}",
        std::env::consts::DLL_EXTENSION,
    ));
    let executable = workspace.path().join("c_abi_driver");
    let cache = workspace.path().join("cache");
    let target = CompilationTarget::host();
    let toolchain_root = workspace.path().join("toolchain");
    let toolchain_libs = toolchain_root
        .join(VERSION)
        .join("toolchains")
        .join(target.margarine_target_triple())
        .join("libs");
    fs::create_dir_all(&toolchain_libs).unwrap();
    for entry in fs::read_dir(resource::toolchain_libs_path(target)).unwrap() {
        let entry = entry.unwrap();
        if entry.metadata().unwrap().is_file() {
            fs::copy(entry.path(), toolchain_libs.join(entry.file_name())).unwrap();
        }
    }
    let helper_object = toolchain_libs.join("c_abi_helpers.o");
    let compile_helper = Command::new("clang")
        .args([
            "-target",
            &target.c_target_triple(),
            "-fPIC",
            "-c",
            helper.to_str().unwrap(),
            "-o",
            helper_object.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        compile_helper.status.success(),
        "C ABI helper failed to compile:\n{}",
        String::from_utf8_lossy(&compile_helper.stderr),
    );


    let build = Command::new(env!("CARGO_BIN_EXE_margarine"))
        .args([
            "build",
            "--shared",
            "--cache",
            cache.to_str().unwrap(),
            "--output",
            dylib.to_str().unwrap(),
            source.to_str().unwrap(),
        ])
        .current_dir(workspace.path())
        .env("MARGARINE_PRELUDE", format!("fixture={}", prelude.display()))
        .env("MARGARINE_TOOLCHAIN_DIR", &toolchain_root)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "Margarine C ABI fixture failed to build:\n{}",
        String::from_utf8_lossy(&build.stderr),
    );

    let clang = Command::new("clang")
        .args([
            driver.to_str().unwrap(),
            dylib.to_str().unwrap(),
            "-Wl,-rpath",
            workspace.path().to_str().unwrap(),
            "-o",
            executable.to_str().unwrap(),
        ])
        .current_dir(workspace.path())
        .output()
        .unwrap();
    assert!(
        clang.status.success(),
        "C ABI driver failed to link:\n{}",
        String::from_utf8_lossy(&clang.stderr),
    );

    let run = Command::new(&executable)
        .env("DYLD_LIBRARY_PATH", workspace.path())
        .env("LD_LIBRARY_PATH", workspace.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "C ABI driver failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        "All C ABI bidirectional tests passed successfully!",
    );
}


#[test]
fn c_abi_codegen_supports_all_targets() {
    let workspace = tempfile::tempdir().unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/native");
    let source_path = fixtures.join("c_abi_cross_target.mar");
    let oracle = fixtures.join("c_abi_oracle.c");
    let resource_dir = Command::new("clang")
        .arg("-print-resource-dir")
        .output()
        .unwrap();
    assert!(
        resource_dir.status.success(),
        "failed to locate Clang resource directory:\n{}",
        String::from_utf8_lossy(&resource_dir.stderr),
    );
    let resource_include =
        PathBuf::from(String::from_utf8_lossy(&resource_dir.stdout).trim())
            .join("include");
    let targets = [
        CompilationTarget::Arm64AppleDarwin,
        CompilationTarget::X86_64UnknownLinuxGnu,
        CompilationTarget::Aarch64UnknownLinuxGnu,
        CompilationTarget::Wasm32UnknownUnknown,
    ];

    for target in targets {
        let arena = Arena::new();
        let mut compiler = Compiler::new(&arena);
        let file = FileData::open(&source_path, &mut compiler.string_map).unwrap();
        let entry = compiler.string_map.get(file.name()).to_string();
        compiler.files.register(file);
        let output = workspace.path().join(target.margarine_target_triple());
        let settings = CompilationSettings {
            compilation_target: target,
            preludes: vec![],
            entry,
            output: output.to_string_lossy().into_owned(),
            cache: workspace.path().join("cache"),
            arena: &arena,
            tests: false,
            shared: true,
        };

        let mut result = compiler.run(&settings).unwrap();
        let errors = compiler.check(&mut result);
        assert!(
            errors.iter().all(|files| files.iter().all(Vec::is_empty)),
            "C ABI fixture produced diagnostics for {}",
            target.margarine_target_triple(),
        );
        compiler.codegen(&settings, &mut result, errors);

        let object = PathBuf::from(format!("{}.o", output.display()));
        assert!(
            object.is_file(),
            "C ABI object was not emitted for {}",
            target.margarine_target_triple(),
        );

        let c_target = target.c_target_triple();
        let oracle_object = workspace.path().join(format!(
            "{}.oracle.o",
            target.margarine_target_triple(),
        ));
        let compile_oracle = Command::new("clang")
            .args(["-target", &c_target, "-fPIC", "-nostdinc", "-isystem"])
            .arg(&resource_include)
            .args(["-c", oracle.to_str().unwrap(), "-o"])
            .arg(&oracle_object)
            .output()
            .unwrap();
        assert!(
            compile_oracle.status.success(),
            "C ABI oracle failed to compile for {}:\n{}",
            target.margarine_target_triple(),
            String::from_utf8_lossy(&compile_oracle.stderr),
        );

        let linked = workspace.path().join(format!(
            "{}.linked",
            target.margarine_target_triple(),
        ));
        let mut link = Command::new("clang");
        link.args([
            "-target",
            &c_target,
            "-fuse-ld=lld",
            "-nostdlib",
            "-shared",
        ])
        .arg(&object)
        .arg(&oracle_object);
        if matches!(target, CompilationTarget::Wasm32UnknownUnknown) {
            link.args(["-Wl,--allow-undefined", "-Wl,--no-entry"]);
        }
        let link = link.arg("-o").arg(&linked).output().unwrap();
        assert!(
            link.status.success(),
            "C ABI fixture failed to link for {}:\n{}",
            target.margarine_target_triple(),
            String::from_utf8_lossy(&link.stderr),
        );
    }
}

