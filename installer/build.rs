use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("payload.tar.gz");

    if let Some(payload) = env::var_os("MARGARINE_INSTALL_PAYLOAD") {
        let payload = PathBuf::from(payload);
        fs::copy(&payload, &out).expect("failed to copy installer payload");
        println!("cargo:rerun-if-changed={}", payload.display());
    } else {
        fs::write(&out, []).expect("failed to create installer placeholder payload");
    }

    println!("cargo:rerun-if-env-changed=MARGARINE_INSTALL_PAYLOAD");
    println!("cargo:rerun-if-env-changed=MARGARINE_INSTALL_VERSION");
    println!("cargo:rerun-if-env-changed=MARGARINE_INSTALL_TARGET");
    println!(
        "cargo:rustc-env=MARGARINE_INSTALL_VERSION={}",
        env::var("MARGARINE_INSTALL_VERSION").unwrap_or_else(|_| "0.0.0".to_string())
    );
    println!(
        "cargo:rustc-env=MARGARINE_INSTALL_TARGET={}",
        env::var("MARGARINE_INSTALL_TARGET").unwrap_or_else(|_| "unknown".to_string())
    );
}
