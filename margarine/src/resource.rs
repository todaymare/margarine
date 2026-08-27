use std::{env, fs, io, path::{Path, PathBuf}};

use semver::Version;

use crate::{version::directory_version, CompilationTarget, VERSION};

const TOOLCHAIN_DIR_ENV: &str = "MARGARINE_TOOLCHAIN_DIR";

pub fn installation_from_executable(executable: &Path) -> Result<(PathBuf, Version), String> {
    let unmanaged =
        || format!(
            "this margarine executable is not managed by the self-updater\n  executable: {}\n\nUpdate margarine using the method that installed this executable.\nSelf-update requires a versioned installation under ~/.margarine.",
            executable.display(),
        );
    if executable.file_name().is_none_or(|name| name != "margarine") {
        return Err(unmanaged());
    }
    let bin_dir =
        executable.parent()
            .filter(|path| path.file_name().is_some_and(|name| name == "bin"))
            .ok_or_else(&unmanaged)?;
    let version_dir =
        bin_dir.parent()
            .ok_or_else(&unmanaged)?;
    let version =
        version_dir.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(&unmanaged)?;
    let version =
        directory_version(version)
            .map_err(|_| unmanaged())?;
    let root =
        version_dir.parent()
            .ok_or_else(&unmanaged)?;
    Ok((root.to_path_buf(), version))
}


pub fn current_installation() -> Result<(PathBuf, Version), String> {
    let executable =
        env::current_exe()
            .and_then(fs::canonicalize)
            .map_err(|error| format!("cannot get current executable: {error}"))?;
    installation_from_executable(&executable)
}




pub fn toolchain_version_dir() -> PathBuf {
    if env::var_os(TOOLCHAIN_DIR_ENV).is_none() {
        if let Ok((root, version)) = current_installation() {
            return root.join(version.to_string());
        }
    }
    toolchain_root().join(VERSION)
}


/// Returns the compiler-versioned directory containing target-specific
/// runtime archives.
pub fn toolchain_libs_path(target: CompilationTarget) -> PathBuf {
    toolchain_version_dir()
        .join("toolchains")
        .join(target.margarine_target_triple())
        .join("libs")
}


/// Returns every regular file in the target-specific runtime library directory.
pub(crate) fn toolchain_link_files(target: CompilationTarget) -> io::Result<Vec<PathBuf>> {

    let mut files = Vec::new();
    for entry in fs::read_dir(toolchain_libs_path(target))? {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            files.push(entry.path());
        }
    }
    files.sort();
    Ok(files)
}


fn toolchain_root() -> PathBuf {
    if let Some(path) = env::var_os(TOOLCHAIN_DIR_ENV) {
        return PathBuf::from(path);
    }
    if let Ok((root, _)) = current_installation() {
        return root;
    }

    env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".margarine"))
        .unwrap_or_else(|| PathBuf::from(".margarine"))
}


