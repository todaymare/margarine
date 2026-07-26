use std::path::PathBuf;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const MAPPINGS : &[(&str, &str)] = &[
    ("arm64-apple-darwin", "aarch64-apple-darwin"),
];


pub fn path_lib(target: &str) -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap()
        .join(".local")
        .join("lib")
        .join("margarine")
        .join(VERSION)
        .join(target)
}


pub fn path_bin() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap()
        .join(".local")
        .join("bin")
}


pub fn path_share() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap()
        .join(".local")
        .join("share")
        .join("margarine")
        .join(VERSION)
}


pub fn path_cache() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap()
        .join(".cache")
        .join("margarine")
        .join(VERSION)
}


pub fn executable_name() -> &'static str {
    if cfg!(windows) { "margarine.exe" } else { "margarine" }
}

pub fn static_library_name() -> &'static str {
    if cfg!(windows) { "margarine_rt.lib" } else { "libmargarine_rt.a" }
}

pub fn installer_name() -> &'static str {
    if cfg!(windows) { "margarine-installer.exe" } else { "margarine-installer" }
}


