use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    os::unix::fs::{self as unix_fs, PermissionsExt as _},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
};

use fs2::FileExt as _;

use sha2::{Digest, Sha256};


const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn versioned_archive(script: &str) -> Vec<u8> {
    let mut compressed = Vec::new();
    {
        let encoder = flate2::write::GzEncoder::new(
            &mut compressed,
            flate2::Compression::default(),
        );
        let mut archive = tar::Builder::new(encoder);
        let contents = format!("#!/bin/sh\n{script}\n");
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        archive.append_data(&mut header, "margarine", contents.as_bytes()).unwrap();
        archive.into_inner().unwrap().finish().unwrap();
    }
    compressed
}


fn toolchain_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut compressed = Vec::new();
    {
        let encoder = flate2::write::GzEncoder::new(
            &mut compressed,
            flate2::Compression::default(),
        );
        let mut archive = tar::Builder::new(encoder);
        for (path, contents) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            archive.append_data(&mut header, path, *contents).unwrap();
        }
        archive.into_inner().unwrap().finish().unwrap();
    }
    compressed
}


fn toolchain_server(
    archive: Vec<u8>,
    expected_checksum: String,
    release_version: &str,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let target = "wasm32-unknown-unknown";
    let asset_name = format!("margarine-toolchain-{target}.tar.gz");
    let release_path = format!("/releases/tags/v{release_version}");
    let release = serde_json::json!({
        "tag_name": format!("v{release_version}"),
        "name": format!("margarine {release_version}"),
        "html_url": "https://example.invalid/release",
        "assets": [
            {
                "name": asset_name,
                "size": archive.len(),
                "browser_download_url": format!("http://{address}/toolchain"),
            },
            {
                "name": format!("{asset_name}.sha256"),
                "size": expected_checksum.len(),
                "browser_download_url": format!("http://{address}/toolchain.sha256"),
            },
        ],
    }).to_string().into_bytes();

    let server = thread::spawn(move || {
        for stream in listener.incoming().take(3) {
            let mut stream = stream.unwrap();
            let mut request = [0u8; 2048];
            let read = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..read]);
            let path = request.split_whitespace().nth(1).unwrap();
            let (status, body): (&str, &[u8]) =
                if path == release_path {
                    ("200 OK", &release)
                } else {
                    match path {
                        "/toolchain" => ("200 OK", &archive),
                        "/toolchain.sha256" => ("200 OK", expected_checksum.as_bytes()),
                        _ => ("404 Not Found", b""),
                    }
                };
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len(),
            ).unwrap();
            stream.write_all(body).unwrap();
        }
    });

    (format!("http://{address}/releases"), server)
}


fn release_server(
    archive: Vec<u8>,
    omitted_toolchains: &[&str],
    checksumless_toolchains: &[&str],
    checksum: Option<&str>,
    request_count: usize,
) -> (String, thread::JoinHandle<()>) {
    release_server_for(
        "0.2.0",
        archive,
        omitted_toolchains,
        checksumless_toolchains,
        checksum,
        request_count,
    )
}


fn release_server_for(
    version: &str,
    archive: Vec<u8>,
    omitted_toolchains: &[&str],
    checksumless_toolchains: &[&str],
    checksum: Option<&str>,
    request_count: usize,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let checksum =
        checksum.map(str::to_string)
            .unwrap_or_else(|| hex::encode(Sha256::digest(&archive)));
    let asset_name = format!("margarine-{}.tar.gz", env!("MARGARINE_TARGET"));
    let mut assets = vec![
        serde_json::json!({
            "name": asset_name,
            "size": archive.len(),
            "browser_download_url": format!("http://{address}/archive"),
        }),
        serde_json::json!({
            "name": format!("{asset_name}.sha256"),
            "size": checksum.len(),
            "browser_download_url": format!("http://{address}/checksum"),
        }),
    ];
    let runtime = toolchain_archive(&[("libs/runtime.a", b"runtime")]);
    let runtime_checksum = hex::encode(Sha256::digest(&runtime));
    let mut responses = vec![
        ("/archive".to_string(), archive),
        ("/checksum".to_string(), checksum.into_bytes()),
    ];
    let mut targets = vec![env!("MARGARINE_TARGET"), "wasm32-unknown-unknown"];
    targets.sort();
    targets.dedup();
    for target in targets {
        if omitted_toolchains.contains(&target) {
            continue;
        }
        let name = format!("margarine-toolchain-{target}.tar.gz");
        let archive_path = format!("/toolchain/{target}");
        let checksum_path = format!("{archive_path}.sha256");
        assets.push(serde_json::json!({
            "name": name,
            "size": runtime.len(),
            "browser_download_url": format!("http://{address}{archive_path}"),
        }));
        responses.push((archive_path, runtime.clone()));
        if !checksumless_toolchains.contains(&target) {
            assets.push(serde_json::json!({
                "name": format!("{name}.sha256"),
                "size": runtime_checksum.len(),
                "browser_download_url": format!("http://{address}{checksum_path}"),
            }));
            responses.push((checksum_path, runtime_checksum.as_bytes().to_vec()));
        }
    }
    let release = serde_json::json!({
        "tag_name": format!("v{version}"),
        "name": format!("margarine {version}"),
        "published_at": "2026-08-23T00:00:00Z",
        "body": "release",
        "html_url": "https://example.invalid/release",
        "assets": assets,
    }).to_string().into_bytes();
    let latest_path = "/releases/latest".to_string();
    let version_path = format!("/releases/tags/v{version}");
    responses.push((latest_path, release.clone()));
    responses.push((version_path, release));

    let server = thread::spawn(move || {
        for stream in listener.incoming().take(request_count) {
            let mut stream = stream.unwrap();
            let mut request = [0u8; 2048];
            let read = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..read]);
            let path = request.split_whitespace().nth(1).unwrap();
            let response = responses.iter().find(|(candidate, _)| candidate == path);
            let (status, body): (&str, &[u8]) =
            match response {
                Some((_, body)) => ("200 OK", body),
                None => ("404 Not Found", b""),
            };
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len(),
            ).unwrap();
            stream.write_all(body).unwrap();
        }
    });

    (format!("http://{address}/releases"), server)
}


fn api_response_server(
    status: &'static str,
    body: &[u8],
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let body = body.to_vec();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len(),
        ).unwrap();
        stream.write_all(&body).unwrap();
    });
    (format!("http://{address}/releases"), server)
}


fn install_binary(root: &Path, version: &str) -> PathBuf {
    let physical = root.join(version).join("bin/margarine");
    fs::create_dir_all(physical.parent().unwrap()).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_margarine"), &physical).unwrap();
    fs::set_permissions(&physical, fs::Permissions::from_mode(0o755)).unwrap();
    physical
}

fn install_managed_fixture(root: &Path) -> PathBuf {
    install_binary(root, CURRENT_VERSION);
    fs::create_dir_all(root.join(CURRENT_VERSION).join("toolchains").join(env!("MARGARINE_TARGET")),).unwrap();
    fs::create_dir_all(root.join(CURRENT_VERSION).join("toolchains/wasm32-unknown-unknown"),).unwrap();
    fs::create_dir_all(
        root.join(CURRENT_VERSION).join("toolchains/not-a-target"),
    ).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    unix_fs::symlink(
        format!("../{CURRENT_VERSION}/bin/margarine"),
        root.join("bin/margarine"),
    ).unwrap();
    root.join("bin/margarine")
}

fn run_update(
    active: &Path,
    api: String,
    server: thread::JoinHandle<()>,
    log: &Path,
    input: &[u8],
) -> Output {
    let mut child = Command::new(active)
        .arg("update")
        .env("MARGARINE_RELEASES_API", api)
        .env("MARGARINE_TEST_LOG", log)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let output = child.wait_with_output().unwrap();
    server.join().unwrap();
    output
}


#[test]
fn toolchain_add_downloads_verifies_and_atomically_installs_runtime_libraries() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let target_dir =
        dir.path().join(CURRENT_VERSION).join("toolchains/wasm32-unknown-unknown");
    fs::remove_dir_all(&target_dir).unwrap();
    let archive = toolchain_archive(&[
        ("libs/libcore.a", b"core runtime"),
        ("libs/libstd.a", b"standard runtime"),
    ]);
    let checksum = hex::encode(Sha256::digest(&archive)).to_ascii_uppercase();
    let (api, server) = toolchain_server(archive, checksum, CURRENT_VERSION);

    let output = Command::new(&active)
        .args(["toolchain", "add", "wasm32-unknown-unknown"])
        .env("MARGARINE_RELEASES_API", api)
        .output()
        .unwrap();
    server.join().unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(fs::read(target_dir.join("libs/libcore.a")).unwrap(), b"core runtime");
    assert_eq!(fs::read(target_dir.join("libs/libstd.a")).unwrap(), b"standard runtime");
    assert!(!fs::read_dir(target_dir.parent().unwrap()).unwrap().any(|entry| {
        entry.unwrap().file_name().to_string_lossy().starts_with(".staging-")
    }));

    let repeated = Command::new(&active)
        .args(["toolchain", "add", "wasm32-unknown-unknown"])
        .output()
        .unwrap();
    assert!(repeated.status.success());
    assert!(
        String::from_utf8_lossy(&repeated.stdout).contains("already installed"),
        "stdout:\n{}",
        String::from_utf8_lossy(&repeated.stdout),
    );
}

#[test]
fn toolchain_add_uses_the_managed_version_directory_for_staged_binaries() {
    let dir = tempfile::tempdir().unwrap();
    let version = "0.2.0";
    let active = install_binary(dir.path(), version);
    let target_dir = dir.path().join(version).join("toolchains/wasm32-unknown-unknown");
    let archive = toolchain_archive(&[("libs/libstd.a", b"standard runtime")]);
    let checksum = hex::encode(Sha256::digest(&archive));
    let (api, server) = toolchain_server(archive, checksum, version);

    let output = Command::new(&active)
        .args(["toolchain", "add", "wasm32-unknown-unknown"])
        .env("MARGARINE_RELEASES_API", format!("{api}/"))
        .output()
        .unwrap();
    server.join().unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(
        fs::read(target_dir.join("libs/libstd.a")).unwrap(),
        b"standard runtime",
    );
}


#[test]
fn toolchain_add_rejects_release_metadata_for_another_version() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let target_dir =
        dir.path().join(CURRENT_VERSION).join("toolchains/wasm32-unknown-unknown");
    fs::remove_dir_all(&target_dir).unwrap();
    let release = serde_json::json!({
        "tag_name": "v0.2.0",
        "name": null,
        "published_at": null,
        "body": null,
        "html_url": "https://example.invalid/release",
        "assets": [],
    }).to_string();
    let (api, server) = api_response_server("200 OK", release.as_bytes());

    let output = Command::new(&active)
        .args(["toolchain", "add", "wasm32-unknown-unknown"])
        .env("MARGARINE_RELEASES_API", api)
        .output()
        .unwrap();
    server.join().unwrap();

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!(
            "release metadata is for v0.2.0, but version {CURRENT_VERSION} was requested",
        )),
        "{stderr}",
    );
    assert!(!target_dir.exists());
}


#[test]
fn toolchain_add_rejects_a_bad_checksum_without_installing_any_files() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let target_dir =
        dir.path().join(CURRENT_VERSION).join("toolchains/wasm32-unknown-unknown");
    fs::remove_dir_all(&target_dir).unwrap();
    let archive = toolchain_archive(&[("libs/libstd.a", b"standard runtime")]);
    let (api, server) = toolchain_server(archive, "0".repeat(64), CURRENT_VERSION);

    let output = Command::new(&active)
        .args(["toolchain", "add", "wasm32-unknown-unknown"])
        .env("MARGARINE_RELEASES_API", api)
        .output()
        .unwrap();
    server.join().unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("checksum mismatch"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(!target_dir.exists());
}


#[test]
fn toolchain_add_rejects_an_archive_without_runtime_libraries() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let target_dir =
        dir.path().join(CURRENT_VERSION).join("toolchains/wasm32-unknown-unknown");
    fs::remove_dir_all(&target_dir).unwrap();
    let archive = toolchain_archive(&[("README", b"not a runtime library")]);
    let checksum = hex::encode(Sha256::digest(&archive));
    let (api, server) = toolchain_server(archive, checksum, CURRENT_VERSION);

    let output = Command::new(&active)
        .args(["toolchain", "add", "wasm32-unknown-unknown"])
        .env("MARGARINE_RELEASES_API", api)
        .output()
        .unwrap();
    server.join().unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid toolchain"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(!target_dir.exists());
}


#[test]
fn updater_rejects_an_unmanaged_executable_before_contacting_the_release_api() {
    let dir = tempfile::tempdir().unwrap();
    let active = dir.path().join("margarine");
    fs::copy(env!("CARGO_BIN_EXE_margarine"), &active).unwrap();
    let (api, server) =
        release_server(versioned_archive("exit 8"), &[], &[], None, 0);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"",
        );

    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "not managed by the self-updater",
        ),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
}


#[test]
fn updater_treats_a_missing_latest_release_as_a_successful_check() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let (api, server) = api_response_server("404 Not Found", b"");

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"",
        );

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("No releases yet"),
        "stdout:\n{}",
        String::from_utf8_lossy(&output.stdout),
    );
}


#[test]
fn updater_reports_release_api_failures_without_parsing_the_response() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let (api, server) = api_response_server("500 Internal Server Error", b"not release JSON");

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"",
        );

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("update check failed"), "{stderr}");
    assert!(!stderr.contains("could not parse release info"), "{stderr}");
}


#[test]
fn updater_rejects_a_release_tag_with_more_than_one_v_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let (api, server) =
        api_response_server(
            "200 OK",
            br#"{
                "tag_name":"vv0.2.0",
                "name":"Malformed release",
                "html_url":"https://example.invalid/release",
                "assets":[]
            }"#,
        );

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"",
        );

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!(
            "cannot compare installed version {CURRENT_VERSION} with release version v0.2.0",
        )),
        "{stderr}",
    );
}

#[test]
fn updater_treats_a_stable_release_as_newer_than_a_prerelease() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let (api, server) =
        release_server_for(
            "0.1.0",
            versioned_archive("exit 8"),
            &[],
            &[],
            None,
            1,
        );

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"n\n",
        );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("{CURRENT_VERSION} -> 0.1.0")),
        "{stdout}",
    );
    assert!(stdout.contains("Install update? [y/N]"), "{stdout}");
    assert_eq!(
        fs::read_link(dir.path().join("bin/margarine")).unwrap(),
        Path::new(&format!("../{CURRENT_VERSION}/bin/margarine")),
    );
    assert!(!dir.path().join("0.1.0").exists());
}


#[test]
fn updater_compares_version_components_numerically() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_binary(dir.path(), "0.2.0");
    let (api, server) =
        release_server_for(
            "0.10.0",
            versioned_archive("exit 8"),
            &[],
            &[],
            None,
            1,
        );

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"n\n",
        );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.2.0 -> 0.10.0"), "{stdout}");
    assert!(stdout.contains("Install update? [y/N]"), "{stdout}");
    assert!(!dir.path().join("0.10.0").exists());
}


#[test]
fn updater_uses_the_managed_version_directory_as_the_current_version() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_binary(dir.path(), "0.2.0");
    let (api, server) =
        release_server(versioned_archive("exit 8"), &[], &[], None, 1);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"",
        );

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("margarine is up to date"), "{stdout}");
    assert!(stdout.contains("Current version: 0.2.0"), "{stdout}");
}


#[test]
fn updater_rejects_an_existing_version_before_downloading_assets() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    fs::create_dir(dir.path().join("0.2.0")).unwrap();
    let (api, server) =
        release_server(versioned_archive("exit 8"), &[], &[], None, 1);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"",
        );

    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "refusing to overwrite existing installation",
        ),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("Install update?"),
        "stdout:\n{}",
        String::from_utf8_lossy(&output.stdout),
    );
}


#[test]
fn updater_holds_the_installation_lock_before_offering_a_download() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let (api, server) =
        release_server(versioned_archive("exit 8"), &[], &[], None, 1);
    let mut child =
        Command::new(&active)
            .arg("update")
            .env("MARGARINE_RELEASES_API", api)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut dialogue = Vec::new();
    let prompt = b"Install update? [y/N] ";
    while !dialogue.ends_with(prompt) {
        let mut byte = [0];
        if stdout.read(&mut byte).unwrap() == 0 {
            panic!(
                "updater exited before prompting:\n{}",
                String::from_utf8_lossy(&dialogue),
            );
        }
        dialogue.push(byte[0]);
    }

    let contender =
        fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.path().join("update.lock"))
            .unwrap();
    let error = contender.try_lock_exclusive().unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);

    stdin.write_all(b"n\n").unwrap();
    drop(stdin);
    stdout.read_to_end(&mut dialogue).unwrap();
    assert!(child.wait().unwrap().success());
    server.join().unwrap();
    contender.try_lock_exclusive().unwrap();
    contender.unlock().unwrap();
}


#[test]
fn updater_rejects_a_malformed_checksum_before_downloading_the_archive() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let (api, server) =
        release_server(versioned_archive("exit 8"), &[], &[], Some("not-a-digest"), 2);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"y\n",
        );

    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "expected 64 hexadecimal digits",
        ),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(!dir.path().join("0.2.0").exists());
    assert!(!fs::read_dir(dir.path()).unwrap().any(|entry| {
        entry.unwrap().file_name().to_string_lossy().starts_with(".staging-")
    }));
}


#[test]
fn updater_stages_toolchains_before_atomically_activating_the_version() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let script =
        "case \"$1\" in\n\
           --version) echo 'margarine 0.2.0' ;;\n\
           toolchain) echo \"$*\" >> \"$MARGARINE_TEST_LOG\" ;;\n\
           *) exit 8 ;;\n\
         esac";
    let (api, server) = release_server(versioned_archive(script), &[], &[], None, 7);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("invocations"),
            b"y\n",
        );

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Install update? [y/N]"),
        "stdout:\n{}",
        String::from_utf8_lossy(&output.stdout),
    );
    assert_eq!(
        fs::read_link(dir.path().join("bin/margarine")).unwrap(),
        Path::new("../0.2.0/bin/margarine"),
    );
    assert!(dir.path().join(CURRENT_VERSION).join("bin/margarine").exists());
    assert!(dir.path().join("0.2.0/bin/margarine").exists());
    assert!(!fs::read_dir(dir.path()).unwrap().any(|entry| {
        entry.unwrap().file_name().to_string_lossy().starts_with(".staging-")
    }));

    let mut installed_targets = vec![
        env!("MARGARINE_TARGET").to_string(),
        "wasm32-unknown-unknown".to_string(),
    ];
    installed_targets.sort();
    installed_targets.dedup();
    for target in installed_targets {
        assert_eq!(
            fs::read(
                dir.path()
                    .join("0.2.0/toolchains")
                    .join(target)
                    .join("libs/runtime.a"),
            ).unwrap(),
            b"runtime",
        );
    }
    assert!(
        !dir.path().join("invocations").exists(),
        "toolchains must be installed in-process",
    );
}


#[test]
fn updater_prompts_for_each_missing_toolchain_and_continues_without_them() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let script =
        "case \"$1\" in\n\
           --version) echo 'margarine 0.2.0' ;;\n\
           toolchain) echo \"$*\" >> \"$MARGARINE_TEST_LOG\" ;;\n\
           *) exit 8 ;;\n\
         esac";
    let omitted = [env!("MARGARINE_TARGET"), "wasm32-unknown-unknown"];
    let (api, server) =
        release_server(versioned_archive(script), &omitted, &[], None, 3);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("invocations"),
            b"y\ny\n",
        );

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("Continue without").count(), 2, "{stdout}");
    assert!(
        stdout.contains(&format!("Continue without {}? [y/N]", env!("MARGARINE_TARGET"))),
        "{stdout}",
    );
    assert!(
        stdout.contains("Continue without wasm32-unknown-unknown? [y/N]"),
        "{stdout}",
    );
    assert!(!stdout.contains("Install update?"), "{stdout}");
    assert!(
        stdout.contains("unavailable toolchains were not carried forward:"),
        "{stdout}",
    );
    assert!(
        !dir.path().join("invocations").exists(),
        "no toolchain subprocess should have been invoked",
    );
    assert_eq!(
        fs::read_link(dir.path().join("bin/margarine")).unwrap(),
        Path::new("../0.2.0/bin/margarine"),
    );
}


#[test]
fn updater_aborts_before_installation_when_a_missing_toolchain_is_declined() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let omitted = [env!("MARGARINE_TARGET"), "wasm32-unknown-unknown"];
    let (api, server) =
        release_server(versioned_archive("exit 8"), &omitted, &[], None, 1);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"y\nn\n",
        );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("Continue without").count(), 2, "{stdout}");
    assert_eq!(
        fs::read_link(dir.path().join("bin/margarine")).unwrap(),
        Path::new(&format!("../{CURRENT_VERSION}/bin/margarine")),
    );
    assert!(!dir.path().join("0.2.0").exists());
}

#[test]
fn updater_treats_confirmation_with_extra_words_as_decline() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let omitted = [env!("MARGARINE_TARGET"), "wasm32-unknown-unknown"];
    let (api, server) =
        release_server(versioned_archive("exit 8"), &omitted, &[], None, 1);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"yes please\n",
        );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("Continue without").count(), 1, "{stdout}");
    assert!(!stdout.contains("Install update?"), "{stdout}");
    assert_eq!(
        fs::read_link(dir.path().join("bin/margarine")).unwrap(),
        Path::new(&format!("../{CURRENT_VERSION}/bin/margarine")),
    );
    assert!(!dir.path().join("0.2.0").exists());
}


#[test]
fn updater_rejects_a_published_toolchain_without_its_checksum() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let checksumless = ["wasm32-unknown-unknown"];
    let (api, server) =
        release_server(versioned_archive("exit 8"), &[], &checksumless, None, 1);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"",
        );

    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "margarine-toolchain-wasm32-unknown-unknown.tar.gz.sha256",
        ),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(
        fs::read_link(dir.path().join("bin/margarine")).unwrap(),
        Path::new(&format!("../{CURRENT_VERSION}/bin/margarine")),
    );
}


#[test]
fn updater_keeps_the_installed_version_when_rollback_cannot_restore_the_link() {
    let dir = tempfile::tempdir().unwrap();
    let active = install_managed_fixture(dir.path());
    let script =
        "case \"$1\" in\n\
           --version)\n\
             case \"$0\" in\n\
               */.staging-*) echo 'margarine 0.2.0' ;;\n\
               *) chmod 500 \"$(dirname \"$0\")\"; exit 7 ;;\n\
             esac ;;\n\
           toolchain) exit 0 ;;\n\
           *) exit 8 ;;\n\
         esac";
    let (api, server) = release_server(versioned_archive(script), &[], &[], None, 7);

    let output =
        run_update(
            &active,
            api,
            server,
            &dir.path().join("unused-log"),
            b"y\n",
        );

    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("rollback also failed"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(
        fs::read_link(dir.path().join("bin/margarine")).unwrap(),
        Path::new("../0.2.0/bin/margarine"),
    );
    assert!(dir.path().join("0.2.0/bin/margarine").exists());
    fs::set_permissions(dir.path().join("bin"), fs::Permissions::from_mode(0o755)).unwrap();
}


