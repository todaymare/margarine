use std::{env, path::PathBuf};

fn main() {
    let rust_target =
        env::var("TARGET").expect("cargo always sets TARGET for build scripts");
    let margarine_target =
        match rust_target.as_str() {
            "aarch64-apple-darwin" => "arm64-apple-darwin",
            target => target,
        };

    println!("cargo:rustc-env=MARGARINE_TARGET={margarine_target}");

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .expect("cargo always sets CARGO_MANIFEST_DIR"),
    );
    let libraries = manifest_dir.join("../libraries");

    if env::var("PROFILE").as_deref() == Ok("debug") && libraries.is_dir() {
        println!(
            "cargo:rustc-env=MARGARINE_SOURCE_LIBRARY_DIR={}",
            libraries.display(),
        );
    }
}
