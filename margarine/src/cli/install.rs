use std::{env, fs, io::Write as _, path::PathBuf};

use colourful::ColourBrush;
use margarine::{resource::current_installation, CompilationTarget, VERSION};

use super::{
    distribution::release_for_version,
    installation::{CompilerSource, Installation},
    CliError,
    CliResult,
    TICK_GLYPH,
};


pub(super) fn execute(assume_yes: bool) -> CliResult<i32> {
    if let Ok((root, version)) = current_installation() {
        println!(
            "{} margarine {version} is already managed at {}",
            TICK_GLYPH.green().bold(),
            root.display(),
        );
        println!("Run `margarine update` to install a newer release.");
        return Ok(0);
    }

    let home =
        env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| CliError::link("cannot install Margarine because HOME is not set"))?;
    let root = home.join(".margarine");
    let installation = Installation::acquire(root).map_err(CliError::link)?;
    installation.ensure_version_absent(VERSION).map_err(CliError::link)?;
    let active = installation.root().join("bin/margarine");
    match fs::symlink_metadata(&active) {
        Ok(_) => return Err(CliError::link(format!(
            "a managed Margarine installation already exists at {}; run that binary's `update` command",
            active.display(),
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(CliError::link(format!(
            "cannot inspect managed installation {}: {error}",
            active.display(),
        ))),
    }

    let release = release_for_version(VERSION).map_err(CliError::link)?;
    let host = CompilationTarget::host();
    super::toolchain::checked_toolchain_assets(&release, host)
        .map_err(CliError::link)?;

    println!("Install Margarine {VERSION} and the {} toolchain into {}?", host.margarine_target_triple(), installation.root().display());
    if !assume_yes {
        print!("Continue? [y/N] ");
        let _ = std::io::stdout().flush();
        let mut answer = String::new();
        std::io::stdin()
            .read_line(&mut answer)
            .map_err(|error| CliError::link(format!("cannot read installation response: {error}")))?;
        if !super::update::answer_is_yes(&answer) {
            println!("Installation cancelled; no changes were made.");
            return Ok(0);
        }
    }

    let executable =
        env::current_exe()
            .and_then(|path| path.canonicalize())
            .map_err(|error| CliError::link(format!("cannot resolve current executable: {error}")))?;
    installation.install_release(
        VERSION,
        &release,
        CompilerSource::Current(&executable),
        &[host],
    ).map_err(CliError::link)?;

    println!(
        "{} Margarine {VERSION} installed at {}",
        TICK_GLYPH.green().bold(),
        installation.version_path(VERSION).display(),
    );
    let bin_dir = installation.root().join("bin");
    let on_path =
        env::var_os("PATH")
            .map(|path| env::split_paths(&path).any(|entry| entry == bin_dir))
            .unwrap_or(false);
    if !on_path {
        println!("Add Margarine to PATH:");
        println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
    }
    Ok(0)
}
