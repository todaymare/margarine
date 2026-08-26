use std::{fs, path::Path};

use colourful::ColourBrush;
use margarine::{resource::current_installation, CompilationTarget};

use super::{
    distribution::{checked_assets, download_checked_assets, extract_tarball, release_for_version, Asset, Release},
    installation::Installation,
    CliError,
    CliResult,
    TICK_GLYPH,
};


pub(super) fn add(target: CompilationTarget) -> CliResult<i32> {
    let (install_root, current_version) =
        current_installation().map_err(CliError::link)?;
    let installation =
        Installation::acquire(install_root).map_err(CliError::link)?;
    let version_dir = installation.version_path(&current_version);
    let target_name = target.margarine_target_triple();
    let target_dir = version_dir.join("toolchains").join(&target_name);
    if !ensure_target_absent(&target_dir, &target_name).map_err(CliError::link)? {
        println!(
            "{} Toolchain `{target_name}` is already installed",
            TICK_GLYPH.green().bold(),
        );
        return Ok(0);
    }

    let release = release_for_version(&current_version).map_err(CliError::link)?;
    install(&version_dir, target, &release).map_err(CliError::link)?;
    Ok(0)
}


pub(super) fn checked_toolchain_assets(
    release: &Release,
    target: CompilationTarget,
) -> Result<(&Asset, &Asset), String> {
    checked_assets(
        release,
        &format!(
            "margarine-toolchain-{}.tar.gz",
            target.margarine_target_triple(),
        ),
    )
}


pub(super) fn install(
    version_dir: &Path,
    target: CompilationTarget,
    release: &Release,
) -> Result<bool, String> {
    let target_name = target.margarine_target_triple();
    let toolchains_dir = version_dir.join("toolchains");
    let target_dir = toolchains_dir.join(&target_name);
    if !ensure_target_absent(&target_dir, &target_name)? {
        return Ok(false);
    }

    let (archive_asset, checksum_asset) =
        checked_toolchain_assets(release, target)?;
    fs::create_dir_all(&toolchains_dir)
        .map_err(|error| format!("cannot create toolchains directory: {error}"))?;
    let staging =
        tempfile::Builder::new()
            .prefix(".staging-")
            .tempdir_in(&toolchains_dir)
            .map_err(|error| format!(
                "cannot create toolchain staging directory: {error}",
            ))?;
    let mut archive = download_checked_assets(archive_asset, checksum_asset)?;
    extract_tarball(archive.as_file_mut(), staging.path(), None)
        .map_err(|error| format!("extraction failed: {error}"))?;
    let libs_dir = staging.path().join("libs");
    let entries =
        fs::read_dir(&libs_dir)
            .map_err(|error| format!(
                "invalid toolchain `{target_name}`: cannot read {}: {error}",
                libs_dir.display(),
            ))?;
    let mut has_runtime_library = false;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!(
                "invalid toolchain `{target_name}`: cannot inspect `libs`: {error}",
            ))?;
        let kind =
            entry.file_type()
                .map_err(|error| format!(
                    "invalid toolchain `{target_name}`: cannot inspect `libs`: {error}",
                ))?;
        has_runtime_library |= kind.is_file();
    }
    if !has_runtime_library {
        return Err(format!(
            "invalid toolchain `{target_name}`: `libs` contains no runtime libraries",
        ));
    }
    if !ensure_target_absent(&target_dir, &target_name)? {
        return Ok(false);
    }
    let staged_dir = staging.keep();
    if let Err(error) = fs::rename(&staged_dir, &target_dir) {
        let _ = fs::remove_dir_all(&staged_dir);
        return Err(format!("cannot install toolchain `{target_name}`: {error}"));
    }

    println!(
        "{} Installed toolchain `{target_name}`",
        TICK_GLYPH.green().bold(),
    );
    Ok(true)
}


fn ensure_target_absent(target_dir: &Path, target_name: &str) -> Result<bool, String> {
    match fs::symlink_metadata(target_dir) {
        Ok(metadata) if metadata.is_dir() => Ok(false),
        Ok(_) => Err(format!(
            "refusing to overwrite existing toolchain `{target_name}` at {}",
            target_dir.display(),
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(format!(
            "cannot inspect toolchain target {}: {error}",
            target_dir.display(),
        )),
    }
}
