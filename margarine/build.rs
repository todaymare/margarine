fn main() {
    println!(
        "cargo:rustc-env=MARGARINE_TARGET={}",
        std::env::var("TARGET").expect("cargo always sets TARGET for build scripts"),
    );
}
