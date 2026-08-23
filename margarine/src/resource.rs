use std::{env, fs, io, path::PathBuf};

use crate::{CompilationTarget, VERSION};

const TOOLCHAIN_DIR_ENV: &str = "MARGARINE_TOOLCHAIN_DIR";

/// Returns the compiler-versioned directory containing target-specific
/// runtime archives.
pub fn toolchain_libs_path(target: CompilationTarget) -> PathBuf {
    toolchain_root()
        .join(VERSION)
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

    env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".margarine"))
        .unwrap_or_else(|| PathBuf::from(".margarine"))
}
