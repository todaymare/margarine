use std::{env, fs, io, path::{Path, PathBuf}};


use crate::{CompilationTarget, VERSION};

const TOOLCHAIN_DIR_ENV: &str = "MARGARINE_TOOLCHAIN_DIR";
const DEV_LIBRARY_DIR_ENV: &str = "MARGARINE_LIBRARY_DIR";

pub fn installation_from_executable(executable: &Path) -> Result<(PathBuf, String), String> {
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
    let mut components = version.split('.');
    let valid_component =
        |component: Option<&str>| {
            component.is_some_and(|component| {
                !component.is_empty()
                    && component.bytes().all(|byte| byte.is_ascii_digit())
            })
        };
    if !valid_component(components.next())
        || !valid_component(components.next())
        || !valid_component(components.next())
        || components.next().is_some()
    {
        return Err(unmanaged());
    }
    let root =
        version_dir.parent()
            .ok_or_else(&unmanaged)?;
    Ok((root.to_path_buf(), version.to_string()))
}


pub fn current_installation() -> Result<(PathBuf, String), String> {
    let executable =
        env::current_exe()
            .and_then(fs::canonicalize)
            .map_err(|error| format!("cannot get current executable: {error}"))?;
    installation_from_executable(&executable)
}


/// Returns the source-library checkout used by an unmanaged debug build.
pub(crate) fn development_library_root() -> Option<PathBuf> {
    if let Some(path) = env::var_os(DEV_LIBRARY_DIR_ENV) {
        return Some(PathBuf::from(path));
    }
    if !cfg!(debug_assertions) {
        return None;
    }
    option_env!("MARGARINE_SOURCE_LIBRARY_DIR").map(PathBuf::from)
}


pub fn toolchain_version_dir() -> PathBuf {
    if env::var_os(TOOLCHAIN_DIR_ENV).is_none() {
        if let Ok((root, version)) = current_installation() {
            return root.join(version);
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_installation_comes_from_the_executable_layout() {
        let managed = Path::new("/opt/margarine/0.1.0/bin/margarine");
        assert_eq!(
            installation_from_executable(managed).unwrap(),
            (PathBuf::from("/opt/margarine"), "0.1.0".to_string()),
        );

        for unmanaged in [
            Path::new("/opt/margarine"),
            Path::new("/opt/margarine/bin/margarine"),
            Path::new("/opt/margarine/0.1.0/bin/not-margarine"),
        ] {
            let error = installation_from_executable(unmanaged).unwrap_err();
            assert!(error.contains("not managed by the self-updater"));
            assert!(error.contains(&unmanaged.display().to_string()));
        }
    }
}
