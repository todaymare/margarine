use std::{fs::{self, File}, io::{Read, Seek, Write}, path::Path, process::Command};

use colourful::ColourBrush;
use flate2::read::GzDecoder;
use margarine::progress::{byte_progress, ProgressReader, StatusLine};
use semver::Version;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use margarine::version::release_tag_version;

use super::{artifacts::format_bytes, TICK_GLYPH};

const DEFAULT_RELEASES_API: &str = "https://api.daymare.net/margarine/v1/releases";


pub(super) fn release_api_url(path: &str) -> String {
    let base =
        std::env::var("MARGARINE_RELEASES_API")
            .unwrap_or_else(|_| DEFAULT_RELEASES_API.into());
    format!("{}/{path}", base.trim_end_matches('/'))
}

pub(super) fn release_for_version(version: &Version) -> Result<Release, String> {
    let checking = StatusLine::start("Checking release");
    let response =
        reqwest::blocking::Client::new()
            .get(release_api_url(&format!("tags/v{version}")))
            .header("User-Agent", "margarine-cli")
            .send();
    checking.clear();
    let response =
        response
            .map_err(|error| format!("release lookup failed: {error}"))?
            .error_for_status()
            .map_err(|error| format!("release lookup failed: {error}"))?;
    let release: Release =
        serde_json::from_reader(response)
            .map_err(|error| format!("invalid release metadata: {error}"))?;
    let release_version =
        release_tag_version(&release.tag_name)
            .map_err(|error| format!("invalid release version {}: {error}", release.tag_name))?;
    if &release_version != version {
        return Err(format!(
            "release metadata is for {}, but version {version} was requested",
            release.tag_name,
        ));
    }
    Ok(release)
}


pub(super) fn checked_assets<'a>(
    release: &'a Release,
    name: &str,
) -> Result<(&'a Asset, &'a Asset), String> {
    let archive =
        release.assets.iter()
            .find(|asset| asset.name == name)
            .ok_or_else(|| format!("release does not contain `{name}`"))?;
    let checksum_name = format!("{name}.sha256");
    let checksum =
        release.assets.iter()
            .find(|asset| asset.name == checksum_name)
            .ok_or_else(|| format!("release does not contain `{checksum_name}`"))?;
    Ok((archive, checksum))
}


pub(super) fn download_checked_assets(
    archive_asset: &Asset,
    checksum_asset: &Asset,
) -> Result<NamedTempFile, String> {
    println!("Downloading {}", archive_asset.name);
    let (mut checksum_file, _) = download(&checksum_asset.browser_download_url)?;
    let mut expected_checksum = String::new();
    checksum_file.as_file_mut()
        .read_to_string(&mut expected_checksum)
        .map_err(|error| format!("cannot read `{}`: {error}", checksum_asset.name))?;
    let expected_checksum = expected_checksum.trim();
    if expected_checksum.len() != 64
        || !expected_checksum.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(format!(
            "invalid checksum in `{}`: expected 64 hexadecimal digits",
            checksum_asset.name,
        ));
    }

    let (archive, received_checksum) = download(&archive_asset.browser_download_url)?;
    if !received_checksum.eq_ignore_ascii_case(expected_checksum) {
        return Err(format!(
            "checksum mismatch for `{}`\n expected {expected_checksum}\n received {received_checksum}",
            archive_asset.name,
        ));
    }

    Ok(archive)
}

pub(super) fn run_binary_check(binary: &Path) -> Result<(), String> {
    let output = Command::new(binary)
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .output()
        .map_err(|error| format!("cannot execute: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "--version exited with {}",
            output.status.code().unwrap_or(-1),
        ));
    }
    let reported =
        String::from_utf8(output.stdout)
            .map_err(|_| "--version output is not UTF-8".to_string())?;
    if reported.trim().is_empty() {
        return Err("unexpected empty --version output".to_string());
    }

    Ok(())
}

pub(super) fn download(url: &str) -> Result<(NamedTempFile, String), String> {
    let response =
        reqwest::blocking::Client::new()
            .get(url)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|error| format!("download failed: {error}"))?;

    let mut file =
        tempfile::NamedTempFile::new()
            .map_err(|error| format!("cannot create temp file: {error}"))?;
    let mut hasher = Sha256::new();
    let progress = byte_progress(response.content_length());
    progress.set_message("Downloading");
    let mut reader = ProgressReader::new(response, &progress);
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read =
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(error) => {
                    progress.finish();
                    return Err(format!("download interrupted: {error}"));
                },
            };

        if let Err(error) = file.write_all(&buffer[..read]) {
            progress.finish();
            return Err(format!("cannot write temp file: {error}"));
        }

        hasher.update(&buffer[..read]);
    }
    progress.finish();

    file.flush()
        .map_err(|error| format!("cannot flush temp file: {error}"))?;
    file.rewind()
        .map_err(|error| format!("cannot rewind temp file: {error}"))?;

    Ok((file, hex::encode(hasher.finalize())))
}

pub(super) fn extract_archive(tarball: &File, destination: &Path) -> Result<(), String> {
    extract_tarball(tarball, destination, Some(Path::new("margarine")))
}


pub(super) fn extract_tarball(
    tarball: &File,
    destination: &Path,
    required_file: Option<&Path>,
) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("cannot create extraction directory: {error}"))?;

    let status = StatusLine::start("Extracting");
    let extraction =
        (|| {
            let decompressed = GzDecoder::new(tarball);
            let mut archive = tar::Archive::new(decompressed);
            archive.set_overwrite(false);
            let entries =
                archive.entries()
                    .map_err(|error| format!("not a valid tar.gz: {error}"))?;
            let mut extracted_size = 0u64;
            let mut extracted_files = 0usize;
            let mut found_required_file = required_file.is_none();

            for candidate in entries {
                let mut entry =
                    candidate.map_err(|error| format!("cannot read archive entry: {error}"))?;
                let path =
                    entry.path()
                        .map_err(|error| format!("invalid archive path: {error}"))?
                        .into_owned();
                if path.as_os_str().is_empty()
                    || !path.components().all(|part| matches!(part, std::path::Component::Normal(_)))
                {
                    return Err(format!("archive path is not relative: {}", path.display()));
                }

                let entry_type = entry.header().entry_type();
                if !entry_type.is_file() && !entry_type.is_dir() {
                    return Err(format!(
                        "archive entry `{}` is not a regular file or directory",
                        path.display(),
                    ));
                }
                if !entry.unpack_in(destination)
                    .map_err(|error| {
                        format!("cannot extract archive entry `{}`: {error}", path.display())
                    })?
                {
                    return Err(format!("archive path escapes destination: {}", path.display()));
                }
                if entry_type.is_dir() {
                    continue;
                }

                extracted_size =
                    extracted_size.checked_add(entry.size())
                        .ok_or_else(|| "extracted archive size overflowed".to_string())?;
                extracted_files += 1;
                found_required_file |= required_file == Some(path.as_path());
            }

            if extracted_files == 0 {
                return Err("archive does not contain any regular files".to_string());
            }

            if let Some(required_file) = required_file {
                if !found_required_file {
                    return Err(format!(
                        "archive does not contain `{}`",
                        required_file.display(),
                    ));
                }
            }
            Ok((extracted_files, extracted_size))
        })();
    status.clear();
    let (extracted_files, extracted_size) = extraction?;

    println!(
        "{} Extracted {} files ({})",
        TICK_GLYPH.green().bold(),
        extracted_files,
        format_bytes(extracted_size),
    );
    Ok(())
}

#[derive(serde::Deserialize)]
pub(super) struct Release {
    pub(super) tag_name: String,
    pub(super) name: Option<String>,
    pub(super) published_at: Option<String>,
    pub(super) body: Option<String>,
    pub(super) html_url: String,
    #[serde(default)]
    pub(super) assets: Vec<Asset>,
}

#[derive(serde::Deserialize)]
pub(super) struct Asset {
    #[allow(dead_code)]
    pub(super) name: String,
    pub(super) size: u64,
    pub(super) browser_download_url: String,
}



#[cfg(test)]
mod tests {
    use std::{
        io::{Seek, SeekFrom},
        os::unix::fs::PermissionsExt as _,
    };

    use tempfile::NamedTempFile;

    use super::*;

    fn write_executable(path: &Path, script: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn archive(entries: &[(&str, &[u8])]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        {
            let encoder = flate2::write::GzEncoder::new(
                file.as_file_mut(),
                flate2::Compression::default(),
            );
            let mut archive = tar::Builder::new(encoder);
            for (path, contents) in entries {
                let mut header = tar::Header::new_gnu();
                header.set_size(contents.len() as u64);
                header.set_mode(0o755);
                header.set_cksum();
                archive.append_data(&mut header, path, *contents).unwrap();
            }
            archive.into_inner().unwrap().finish().unwrap();
        }
        file.seek(SeekFrom::Start(0)).unwrap();
        file
    }

    #[test]
    fn binary_check_accepts_any_nonempty_version_output() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("margarine");
        write_executable(&binary, "echo 'custom build'");

        run_binary_check(&binary).unwrap();
    }

    #[test]
    fn binary_check_rejects_empty_version_output() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("margarine");
        write_executable(&binary, "true");

        let error = run_binary_check(&binary).unwrap_err();

        assert!(error.contains("empty --version output"));
    }

    #[test]
    fn binary_check_rejects_a_nonzero_exit() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("margarine");
        write_executable(&binary, "echo 'custom build'\nexit 7");

        let error = run_binary_check(&binary).unwrap_err();

        assert!(error.contains("exited with"));
    }

    #[test]
    fn extraction_unpacks_the_complete_flat_archive_into_bin() {
        let dir = tempfile::tempdir().unwrap();
        let tarball = archive(&[
            ("margarine", b"new binary"),
            ("margarine-helper", b"helper"),
            ("support/config", b"config"),
        ]);
        let destination = dir.path().join("0.2.0/bin");

        extract_archive(tarball.as_file(), &destination).unwrap();

        assert_eq!(fs::read(destination.join("margarine")).unwrap(), b"new binary");
        assert_eq!(fs::read(destination.join("margarine-helper")).unwrap(), b"helper");
        assert_eq!(fs::read(destination.join("support/config")).unwrap(), b"config");
        assert_eq!(
            fs::metadata(destination.join("margarine")).unwrap().permissions().mode() & 0o777,
            0o755,
        );

        let old_layout = archive(&[("bin/margarine", b"nested binary")]);
        let error =
            extract_archive(old_layout.as_file(), &dir.path().join("other-bin"))
                .unwrap_err();
        assert!(error.contains("does not contain `margarine`"));

        let empty = archive(&[]);
        let error =
            extract_tarball(empty.as_file(), &dir.path().join("empty"), None)
                .unwrap_err();
        assert!(error.contains("does not contain any regular files"));
    }

    #[test]
    fn extraction_rejects_duplicate_entries_without_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let tarball = archive(&[
            ("margarine", b"first"),
            ("margarine", b"second"),
        ]);
        let destination = dir.path().join("bin");

        let error = extract_archive(tarball.as_file(), &destination).unwrap_err();

        assert!(error.contains("cannot extract archive entry `margarine`"));
        assert_eq!(fs::read(destination.join("margarine")).unwrap(), b"first");
    }

    #[test]
    fn extraction_rejects_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let mut tarball = NamedTempFile::new().unwrap();
        {
            let encoder = flate2::write::GzEncoder::new(
                tarball.as_file_mut(),
                flate2::Compression::default(),
            );
            let mut archive = tar::Builder::new(encoder);
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            header.set_mode(0o755);
            header.set_link_name("../../outside").unwrap();
            header.set_cksum();
            archive.append_data(&mut header, "margarine", &[][..]).unwrap();
            archive.into_inner().unwrap().finish().unwrap();
        }
        tarball.seek(SeekFrom::Start(0)).unwrap();

        let error =
            extract_archive(tarball.as_file(), &dir.path().join("bin"))
                .unwrap_err();

        assert!(error.contains("not a regular file or directory"));
        assert!(!dir.path().join("outside").exists());
    }
}
