use std::{cmp::Ordering, fs::File, io::{Read, Seek, Write}, os::unix::fs::PermissionsExt as _, path::Path, process::Command, time::Duration};
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use sti::writeln;

use colourful::ColourBrush;
use tempfile::NamedTempFile;

use margarine::{TARGET, VERSION, VERSION_INFO};
use crate::{fail, format_bytes, page_if_tty, render_markdown_line, LINK_ERROR, TICK_GLYPH, X_GLYPH};

pub fn cmd_update() -> i32 {
    let api_url = 
    std::env::var("MARGARINE_RELEASES_API")
        .unwrap_or_else(|_| "https://api.daymare.net/margarine/v1/releases/latest".into());
    let fetching = spinner();
    fetching.set_message("checking updates");

    // fetch response

    let response = reqwest::blocking::Client::new()
        .get(api_url)
        .header("User-Agent", "margarine-cli")
        .send();


    fetching.finish_and_clear();

    let response = 
    match response {
        Ok(response) => response,
        Err(error) => {
            fail(LINK_ERROR, format!("update check failed: {error}"))
        },
    };

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        println!(" {} {}", TICK_GLYPH.green(), "no updates available");
        println!("   current version: {VERSION_INFO}");
        return 0;
    }

    let release: Release = 
    match response.text() {
        Ok(text) => {
            match serde_json::from_str(&text) {
                Ok(release) => release,
                Err(error) => fail(LINK_ERROR, format!("could not parse release info: {error}")),
            }
        }
        Err(error) => fail(LINK_ERROR, format!("update check failed: {error}")),
    };

    let latest = release.tag_name.trim_start_matches('v');

    if semver_cmp(VERSION, latest).is_ge() {
        println!(" {} margarine is up to date", TICK_GLYPH.green());
        println!("   current version: {VERSION_INFO}");
        return 0;
    }

    let mut dialogue = String::new();

    println!(
        "   margarine {} {} {}",
        VERSION,
        "->",
        latest.green().bold()
    );

    println!();

    let name = release.name.unwrap_or(VERSION_INFO.into());

    let asset_name = format!("margarine-{TARGET}.tar.gz");
    let asset = release.assets.iter().find(|asset| asset.name == asset_name);
    let Some(asset) = asset
    else {
        println!("{} no binary available for this platform", X_GLYPH.red().bold());
        println!("   expected {}", asset_name.cyan());
        return LINK_ERROR;
    };


    let sums_name = format!("{asset_name}.sha256");
    let sums = release.assets.iter().find(|asset| asset.name == sums_name);
    let Some(checksum) = sums
    else {
        println!("{} release has no checksum for this platform", X_GLYPH.red().bold());
        println!("   expected {}", sums_name.cyan());
        return LINK_ERROR;
    };

    let checksum = {
        let (mut checksum, _) = download(&checksum.browser_download_url);
        let mut buf = String::new();
        checksum.read_to_string(&mut buf)
            .unwrap_or_else(|error| fail(LINK_ERROR, format!("cannot read checksum: {error}")));

        buf.trim().to_string()
    };

    writeln!(&mut dialogue, "{name}", name = name.bold());
    writeln!(&mut dialogue, "download: {}", format_bytes(asset.size).cyan());

    if let Some(date) = release.published_at.as_deref().and_then(|d| d.get(..10)) {
        writeln!(&mut dialogue, "published: {}", date.cyan());
    }

    if let Some(body) = &release.body {
        for line in body.lines() {
            render_markdown_line(&mut dialogue, line);
        }
    }

    writeln!(&mut dialogue, "full changelog:");
    writeln!(&mut dialogue, "  {}", release.html_url.clone().underline());

    page_if_tty(&dialogue);

    println!();
    print!("Install update? [y/N] ");
    let _ = std::io::stdout().flush();

    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer).is_err() {
        return 1;
    }

    if !matches!(&*answer.trim().to_lowercase(), "y" | "yes") {
        return 0;
    }

    // download tar

    let (file, file_checksum) = download(&asset.browser_download_url);
    println!(
        " {} downloaded {} ({})",
        TICK_GLYPH.green().bold(),
        asset.name,
        format_bytes(asset.size)
    );



    // checksum
    
    if file_checksum != checksum {
        println!("{} checksum mismatch", X_GLYPH.red().bold());
        println!("   expected {}", checksum.cyan());
        println!("   received {}", file_checksum.cyan());
        return 1;
    }


    // extract beside the live binary so the install can use atomic renames
    let curr_executable = std::env::current_exe()
        .and_then(std::fs::canonicalize)
        .unwrap_or_else(|error| fail(LINK_ERROR, format!("cannot get current executable: {error}")));
    let executable_dir = curr_executable.parent()
        .unwrap_or_else(|| fail(LINK_ERROR, "current executable has no parent directory"));
    let new_executable = extract(file.as_file(), executable_dir);


    // verify binary

    let verify_binary = spinner();
    verify_binary.set_message("verifying binary (1/2)");


    if let Err(e) = run_binary_check(new_executable.path()) {
        verify_binary.finish_and_clear();
        println!("{} couldn't verify installation", X_GLYPH.red().bold());
        println!("   {e}");
        return 1;
    }

    verify_binary.finish_and_clear();

    println!(
        " {} verified binary (1/2)",
        TICK_GLYPH.green().bold(),
    );


    // atomically replace the current binary, then verify it at its final path
    let verify_binary = spinner();
    verify_binary.set_message("verifying binary (2/2)");

    if let Err(e) = install_update(new_executable, &curr_executable) {
        verify_binary.finish_and_clear();
        println!("{} couldn't verify installation", X_GLYPH.red().bold());
        println!("   {e}");
        return 1;
    }

    verify_binary.finish_and_clear();

    println!(
        " {} verified binary (2/2)",
        TICK_GLYPH.green().bold(),
    );


    println!(
        " {} margarine updated to {}", 
        TICK_GLYPH.green(), 
        release.tag_name.trim_start_matches('v')
    );
    0
}


fn run_binary_check(binary: &Path) -> Result<(), String> {
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

    let version = String::from_utf8_lossy(&output.stdout);
    if version.trim().is_empty() {
        return Err("unexpected empty --version output".to_string());
    }

    Ok(())
}


fn install_update(
    new_executable: NamedTempFile,
    curr_executable: &Path,
) -> Result<(), String> {
    let executable_dir = curr_executable.parent()
        .ok_or_else(|| "current executable has no parent directory".to_string())?;
    let backup = executable_dir.join(".margarine.old");

    match std::fs::remove_file(&backup) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("cannot remove stale backup: {error}")),
    }

    std::fs::rename(curr_executable, &backup)
        .map_err(|error| format!("cannot back up current executable: {error}"))?;

    if let Err(error) = new_executable.persist(curr_executable) {
        return match std::fs::rename(&backup, curr_executable) {
            Ok(()) => Err(format!("cannot install new executable: {}", error.error)),
            Err(restore_error) => Err(format!(
                "cannot install new executable: {}; backup restoration also failed: {restore_error}",
                error.error,
            )),
        };
    }

    if let Err(error) = run_binary_check(curr_executable) {
        return match std::fs::rename(&backup, curr_executable) {
            Ok(()) => Err(format!("installed binary failed verification and was rolled back: {error}")),
            Err(restore_error) => Err(format!(
                "installed binary failed verification: {error}; backup restoration also failed: {restore_error}",
            )),
        };
    }

    if let Err(error) = std::fs::remove_file(&backup) {
        eprintln!("warning: could not remove update backup: {error}");
    }

    Ok(())
}




fn download(url: &str) -> (NamedTempFile, String) {
    let mut response =
    match reqwest::blocking::Client::new().get(url).send() {
        Ok(response) => response.error_for_status()
            .unwrap_or_else(|error| fail(LINK_ERROR, format!("download failed: {error}"))),
        Err(error) => fail(LINK_ERROR, format!("download failed: {error}")),
    };


    let mut file = tempfile::NamedTempFile::new()
        .unwrap_or_else(|error| fail(LINK_ERROR, format!("cannot create temp file: {error}")));
    let mut hasher = Sha256::new();

    with_progress("downloading".into(), response.content_length().unwrap_or(1), 
    |bar| {
        let mut buffer = [0u8; 64 * 1024];

        loop {

            let read = 
            match response.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(error) => fail(LINK_ERROR, format!("download interrupted: {error}")),
            };

            file.write_all(&buffer[..read])
                .unwrap_or_else(|error| fail(LINK_ERROR, format!("cannot write temp file: {error}")));

            hasher.update(&buffer[..read]);

            bar.inc(read as u64);
        }
    });

    let _ = file.flush();
    let _ = file.rewind();

    let hex = hex::encode(hasher.finalize());

    (file, hex)
}


fn extract(tarball: &File, executable_dir: &Path) -> NamedTempFile {
    let spinner = spinner();
    spinner.set_message("extracting entries");
    spinner.tick();

    let mut out = NamedTempFile::new_in(executable_dir)
        .unwrap_or_else(|error| fail(LINK_ERROR, format!("cannot create update file: {error}")));


    let decompressed = GzDecoder::new(tarball);
    let mut archive = tar::Archive::new(decompressed);
    let mut entry = archive
        .entries()
        .unwrap_or_else(|error| {
            spinner.finish_and_clear();
            fail(LINK_ERROR, format!("not a valid tar.gz: {error}"))
        })
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .path()
                .ok()
                .and_then(|p| p.file_name().map(|p| p.to_string_lossy().to_string()))
                .is_some_and(|name| name == "margarine")
        })
        .unwrap_or_else(|| {
            spinner.finish_and_clear();
            fail(LINK_ERROR, "archive does not contain a `margarine` binary")
        });

    spinner.finish_and_clear();
    with_progress(
        "extracting".into(), 
        entry.size(),
        |bar| {
            let mut writer = ProgressWriter {
                writer: &mut out,
                bar,
            };

            std::io::copy(&mut entry, &mut writer)
                .unwrap_or_else(|error| fail(LINK_ERROR, format!("extraction failed: {error}")));
        });

    out.as_file_mut()
        .set_permissions(std::fs::Permissions::from_mode(0o755))
        .unwrap_or_else(|error| fail(LINK_ERROR, format!("cannot set permissions: {error}")));

    println!(
        " {} extracted margarine ({})",
        TICK_GLYPH.green().bold(),
        format_bytes(entry.size())
    );


    out
}



fn with_progress<T>(
    message: String,
    total: u64,
    f: impl FnOnce(&ProgressBar) -> T,
) -> T {
    let pb = ProgressBar::new(total);

    pb.enable_steady_tick(Duration::from_millis(50));
    pb.set_style(progress_style());
    pb.set_message(message);

    let result = f(&pb);

    pb.finish_and_clear();
    result
}


fn spinner() -> ProgressBar {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(50));
    spinner.set_style(indicatif::ProgressStyle::with_template(" {spinner:.green} {msg}")
        .expect("static template is valid"));
    spinner

}


fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template(
        " {spinner:.green} {msg} {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})"
    )
    .unwrap()
    .progress_chars("█▓░")
}


struct ProgressWriter<'a, W: Write> {
    writer: W,
    bar: &'a ProgressBar,
}


impl<'a, W: Write> Write for ProgressWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let result = self.writer.write(buf)?;
        self.bar.inc(result as _);
        Ok(result)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}



#[derive(serde::Deserialize)]
struct Release {
    tag_name: String,
    name: Option<String>,
    published_at: Option<String>,
    body: Option<String>,
    html_url: String,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(serde::Deserialize)]
struct Asset {
    #[allow(dead_code)]
    name: String,
    size: u64,
    browser_download_url: String,
}

fn semver_cmp(left: &str, right: &str) -> Ordering {
    let part = |text: &str, index: usize| -> u64 {
        text.split('.').nth(index)
            .and_then(|part| part.parse().ok())
            .unwrap_or(0)
    };

    for index in 0..3 {
        match part(left, index).cmp(&part(right, index)) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    Ordering::Equal
}


#[cfg(test)]
mod tests {
    use super::*;

    fn write_executable(path: &Path, script: &str) {
        std::fs::write(path, format!("#!/bin/sh\n{script}\n")).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn staged_executable(dir: &Path, script: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new_in(dir).unwrap();
        write!(file, "#!/bin/sh\n{script}\n").unwrap();
        file.flush().unwrap();
        file.as_file_mut()
            .set_permissions(std::fs::Permissions::from_mode(0o755))
            .unwrap();
        file
    }

    #[test]
    fn install_update_atomically_replaces_binary() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("margarine");
        write_executable(&current, "echo 'margarine 0.1.0'");
        let staged = staged_executable(dir.path(), "echo 'margarine 0.2.0'");

        install_update(staged, &current).unwrap();

        run_binary_check(&current).unwrap();
        assert!(!dir.path().join(".margarine.old").exists());
    }

    #[test]
    fn install_update_restores_binary_after_final_check_failure() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("margarine");
        write_executable(&current, "echo 'margarine 0.1.0'");
        let staged = staged_executable(
            dir.path(),
            "case \"$0\" in */margarine) exit 7;; *) echo 'margarine 0.2.0';; esac",
        );

        let error = install_update(staged, &current).unwrap_err();

        assert!(error.contains("was rolled back"));
        run_binary_check(&current).unwrap();
    }

    #[test]
    fn binary_check_rejects_empty_version_output() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("margarine");
        write_executable(&binary, "true");

        let error = run_binary_check(&binary).unwrap_err();

        assert!(error.contains("empty --version output"));
    }
}
