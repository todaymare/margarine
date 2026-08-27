use std::env;

fn main() {
    let rust_target = env::var("TARGET").expect("cargo always sets TARGET for build scripts");
    let margarine_target =
    match rust_target.as_str() {
        "aarch64-apple-darwin" => "arm64-apple-darwin",
        target => target,
    };

    println!("cargo:rustc-env=MARGARINE_TARGET={margarine_target}");
}
