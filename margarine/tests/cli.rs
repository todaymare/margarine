use std::{
    fs::{self, File, OpenOptions},
    os::fd::FromRawFd as _,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use fs2::FileExt;


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
    let prelude = dir.path().join("prelude");
    fs::create_dir(&prelude).unwrap();
    fs::write(prelude.join("lib.mar"), "").unwrap();

    let repository = git2::Repository::init(&prelude).unwrap();
    let mut index = repository.index().unwrap();
    index.add_path(Path::new("lib.mar")).unwrap();
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

