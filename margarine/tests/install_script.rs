use std::{
    env,
    fs,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use sha2::{Digest, Sha256};


fn archive(entries: &[(&str, &[u8], u32)]) -> Vec<u8> {
    let mut compressed = Vec::new();
    {
        let encoder = flate2::write::GzEncoder::new(
            &mut compressed,
            flate2::Compression::default(),
        );
        let mut archive = tar::Builder::new(encoder);
        for (path, contents, mode) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(*mode);
            header.set_cksum();
            archive.append_data(&mut header, path, *contents).unwrap();
        }
        archive.into_inner().unwrap().finish().unwrap();
    }
    compressed
}


fn write_asset(directory: &Path, name: &str, contents: &[u8]) {
    fs::write(directory.join(name), contents).unwrap();
    fs::write(
        directory.join(format!("{name}.sha256")),
        hex::encode(Sha256::digest(contents)),
    ).unwrap();
}


fn install_fixture(compiler: &[u8]) -> (tempfile::TempDir, Output) {
    let fixture = tempfile::tempdir().unwrap();
    let downloads = fixture.path().join("downloads");
    let fake_bin = fixture.path().join("fake-bin");
    let home = fixture.path().join("home");
    fs::create_dir(&downloads).unwrap();
    fs::create_dir(&fake_bin).unwrap();
    fs::create_dir(&home).unwrap();

    let target = env!("MARGARINE_TARGET");
    let compiler_name = format!("margarine-{target}.tar.gz");
    let toolchain_name = format!("margarine-toolchain-{target}.tar.gz");
    write_asset(
        &downloads,
        &compiler_name,
        &archive(&[
            ("margarine", compiler, 0o755),
            ("libsupport.dylib", b"companion", 0o644),
        ]),
    );
    write_asset(
        &downloads,
        &toolchain_name,
        &archive(&[("libs/runtime.a", b"runtime", 0o644)]),
    );

    let curl = fake_bin.join("curl");
    fs::write(
        &curl,
        "#!/bin/sh\noutput=\nprevious=\nfor argument in \"$@\"; do\n    if [ \"$previous\" = --output ]; then output=$argument; fi\n    previous=$argument\n    url=$argument\ndone\ncp \"$FAKE_RELEASE_DIR/${url##*/}\" \"$output\"\n",
    ).unwrap();
    fs::set_permissions(&curl, fs::Permissions::from_mode(0o755)).unwrap();

    let mut paths = vec![fake_bin];
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap()));
    let path = env::join_paths(paths).unwrap();
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .join("scripts/install.sh");
    let output =
        Command::new("sh")
            .arg(script)
            .env("HOME", home)
            .env("PATH", path)
            .env("FAKE_RELEASE_DIR", downloads)
            .env("MARGARINE_RELEASE_DOWNLOAD_URL", "https://example.invalid/download")
            .output()
            .unwrap();

    (fixture, output)
}


#[test]
fn shell_installer_publishes_the_complete_compiler_and_host_toolchain() {
    let compiler = b"#!/bin/sh\n[ \"${1:-}\" = --version ] || exit 2\nprintf '%s\\n' 'margarine 0.2.0'\n";
    let (fixture, output) = install_fixture(compiler);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let root = fixture.path().join("home/.margarine");
    assert_eq!(
        fs::read_link(root.join("bin/margarine")).unwrap(),
        PathBuf::from("../0.2.0/bin/margarine"),
    );
    assert_eq!(
        fs::read(root.join("0.2.0/bin/libsupport.dylib")).unwrap(),
        b"companion",
    );
    assert_eq!(
        fs::read(
            root.join("0.2.0/toolchains")
                .join(env!("MARGARINE_TARGET"))
                .join("libs/runtime.a"),
        ).unwrap(),
        b"runtime",
    );
    assert!(!root.join("install.lock").exists());
}


#[test]
fn shell_installer_rolls_back_when_the_activated_compiler_fails() {
    let compiler = b"#!/bin/sh\nif [ \"${1:-}\" != --version ]; then exit 2; fi\nif [ -L \"$0\" ]; then exit 9; fi\nprintf '%s\\n' 'margarine 0.2.0'\n";
    let (fixture, output) = install_fixture(compiler);

    assert!(!output.status.success());
    let root = fixture.path().join("home/.margarine");
    assert!(!root.join("bin/margarine").exists());
    assert!(!root.join("0.2.0").exists());
    assert!(!root.join("install.lock").exists());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("installed compiler failed its final check"),
    );
}
