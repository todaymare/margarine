use std::{cmp::Ordering, fs, io::Write, path::Path, process::Command};

use colourful::ColourBrush;
use margarine::{progress::StatusLine, resource::current_installation, CompilationTarget, TARGET, VERSION_INFO};
use sti::writeln;

use margarine::version::release_tag_version;

use super::{
    artifacts::format_bytes,
    distribution::{checked_assets, release_api_url, Release},
    installation::Installation,
    CliError, CliResult, TICK_GLYPH,
};

pub(super) fn execute() -> CliResult<i32> {
    let (install_root, current_version) =
        current_installation().map_err(CliError::link)?;
    let installation =
        Installation::acquire(install_root).map_err(CliError::link)?;
    let api_url = release_api_url("latest");
    let fetching = StatusLine::start("Checking updates");
    let response =
        reqwest::blocking::Client::new()
            .get(api_url)
            .header("User-Agent", "margarine-cli")
            .send();
    fetching.clear();

    let response =
        response.map_err(|error| CliError::link(format!("update check failed: {error}")))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        println!("{} No releases yet", TICK_GLYPH.green());
        println!("  Current version: {current_version}");
        return Ok(0);
    }
    let response =
        response.error_for_status()
            .map_err(|error| CliError::link(format!("update check failed: {error}")))?;

    let release: Release =
        serde_json::from_reader(response)
            .map_err(|error| CliError::link(format!("could not parse release info: {error}")))?;

    let release_version =
        release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);
    let latest =
        release_tag_version(&release.tag_name)
            .map_err(|_| CliError::link(format!(
                "cannot compare installed version {current_version} with release version {release_version}",
            )))?;

    let version_order = current_version.cmp_precedence(&latest);
    if version_order != Ordering::Less {
        println!("{} margarine is up to date", TICK_GLYPH.green());
        println!("  Current version: {current_version}");
        return Ok(0);
    }

    installation.ensure_version_absent(&latest).map_err(CliError::link)?;
    let asset_name = format!("margarine-{TARGET}.tar.gz");
    let (asset, checksum_asset) =
        checked_assets(&release, &asset_name).map_err(CliError::link)?;
    let expected_targets =
        installed_targets(&installation.version_path(&current_version))
            .map_err(CliError::link)?;

    let mut targets = Vec::with_capacity(expected_targets.len());
    let mut missing_targets = Vec::new();
    for target in expected_targets {
        let toolchain_asset = format!("margarine-toolchain-{target}.tar.gz");
        if !release.assets.iter().any(|asset| asset.name == toolchain_asset) {
            missing_targets.push(target);
            continue;
        }

        checked_assets(&release, &toolchain_asset).map_err(CliError::link)?;
        targets.push(
            CompilationTarget::try_from(target.as_str())
                .expect("installed_targets only returns supported targets"),
        );
    }

    println!(
        "   margarine {} {} {}",
        current_version,
        "->",
        latest.to_string().green().bold()
    );
    println!();

    let name = release.name.as_deref().unwrap_or(VERSION_INFO);
    let mut dialogue = String::new();
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
    writeln!(&mut dialogue, "  {}", release.html_url.as_str().underline());
    page_if_tty(&dialogue);

    for target in &missing_targets {
        println!();
        if target == TARGET {
            println!(
                "{} release {latest} does not provide the host toolchain",
                "warning:".yellow().bold(),
            );
            println!("  {target}");
            println!();
            println!(
                "The updated compiler may be unable to build or run programs on this machine."
            );
        } else {
            println!(
                "{} release {latest} does not provide an installed toolchain",
                "warning:".yellow().bold(),
            );
            println!("  {target}");
            println!();
            println!("Updating will remove support for this target.");
        }
        print!("Continue without {target}? [y/N] ");
        let _ = std::io::stdout().flush();

        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err() {
            return Ok(1);
        }
        if !answer_is_yes(&answer) {
            return Ok(0);
        }
    }

    if missing_targets.is_empty() {
        println!();
        print!("Install update? [y/N] ");
        let _ = std::io::stdout().flush();

        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err() {
            return Ok(1);
        }

        if !answer_is_yes(&answer) {
            return Ok(0);
        }
    }

    installation.install_release(
        &latest,
        &release,
        asset,
        checksum_asset,
        &targets,
    ).map_err(CliError::link)?;

    println!(
        "{} margarine updated to {}",
        TICK_GLYPH.green(),
        latest,
    );
    if !missing_targets.is_empty() {
        println!(
            "{} unavailable toolchains were not carried forward:",
            "warning:".yellow().bold(),
        );
        for target in missing_targets {
            println!("  {target}");
        }
    }
    Ok(0)
}

pub(super) fn answer_is_yes(answer: &str) -> bool {
    let answer = answer.trim();
    answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes")
}




fn installed_targets(version_dir: &Path) -> Result<Vec<String>, String> {
    let mut targets = vec![TARGET.to_string()];
    let toolchain_dir = version_dir.join("toolchains");
    let entries =
        match fs::read_dir(&toolchain_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(targets),
            Err(error) => return Err(format!("cannot inspect installed toolchains: {error}")),
        };
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("cannot inspect installed toolchains: {error}"))?;
        if !entry.file_type()
            .map_err(|error| format!("cannot inspect installed toolchain: {error}"))?
            .is_dir()
        {
            continue;
        }
        let Some(target) = entry.file_name().to_str().map(str::to_string)
        else {
            continue;
        };
        if CompilationTarget::try_from(target.as_str()).is_ok() {
            targets.push(target);
        }
    }
    targets.sort();
    targets.dedup();
    Ok(targets)
}



/// Renders one GitHub-markdown line for the update dialogue. Deliberately
/// tiny: headings become bold labels, list items become bullets, everything
/// else passes through indented.
fn render_markdown_line(out: &mut String, line: &str) {
    let trimmed = line.trim_start();

    if let Some(heading) = trimmed.strip_prefix('#') {
        let heading = heading.trim_start_matches('#').trim();
        if !heading.is_empty() {
            let _ = writeln!(out, "\n{}", heading.bold());
            return;
        }
    }

    if let Some(item) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        let _ = writeln!(out, "  • {item}");
        return;
    }

    let _ = writeln!(out, "  {line}");
}


/// Pipes `text` through $PAGER (default less) only when stdout is a terminal
/// and the text does not fit on one screen; otherwise prints it directly.
fn page_if_tty(text: &str) {
    let is_tty = unsafe { libc::isatty(libc::STDOUT_FILENO) } == 1;
    if !is_tty {
        print!("{text}");
        return;
    }

    let lines = text.lines().count() as i32;
    let mut winsize = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let rows = unsafe {
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize) == 0 {
            winsize.ws_row
        } else {
            0
        }
    };

    if rows == 0 || lines <= rows.into() {
        print!("{text}");
        return;
    }

    let pager = std::env::var("MARGARINE_PAGER")
        .or_else(|_| std::env::var("PAGER"))
        .unwrap_or_else(|_| "less".to_string());

    let mut child = match Command::new(&pager)
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            print!("{text}");
            return;
        }
    };

    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(text.as_bytes());
    }
    drop(child.stdin.take());
    let _ = child.wait();
}


