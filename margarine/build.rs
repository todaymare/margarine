use std::{
    env,
    fs,
    path::PathBuf,
    process::Command,
};

fn git(args: &[&str]) -> Option<String> {
    let output =
    Command::new("git")
        .args(args)
        .output()
        .ok()?;

    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_owned()
    })
}

fn is_dirty() -> bool {
    Command::new("git")
        .args([
            "status",
            "--porcelain",
            "--untracked-files=no",
        ])
        .output()
        .map(|output| output.status.success() && !output.stdout.is_empty())
        .unwrap_or(false)
}

fn exact_tag() -> Option<String> {
    git(&["describe", "--tags", "--exact-match", "HEAD"])
}

fn rerun_on_git_change() {
    println!("cargo:rerun-if-env-changed=MARGARINE_GIT_HASH");

    let Some(git_dir) = git(&["rev-parse", "--git-dir"]) else {
        return;
    };
    let git_dir = PathBuf::from(git_dir);

    for file in &["HEAD", "packed-refs", "refs/tags"] {
        let path = git_dir.join(file);
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    if let Ok(head) = fs::read_to_string(git_dir.join("HEAD")) {
        if let Some(reference) = head.strip_prefix("ref: ") {
            let ref_path = git_dir.join(reference.trim());
            if ref_path.exists() {
                println!("cargo:rerun-if-changed={}", ref_path.display());
            }
        }
    }
}

fn main() {
    let rust_target = env::var("TARGET").expect("cargo always sets TARGET for build scripts");
    let margarine_target =
    match rust_target.as_str() {
        "aarch64-apple-darwin" => "arm64-apple-darwin",
        target => target,
    };
    println!("cargo:rustc-env=MARGARINE_TARGET={margarine_target}");

    rerun_on_git_change();

    let package_version = env::var("CARGO_PKG_VERSION").unwrap();
    let expected_tag = format!("v{package_version}");

    let hash =
    env::var("MARGARINE_GIT_HASH")
        .ok()
        .or_else(|| git(&["rev-parse", "--short", "HEAD"]));

    let dirty = is_dirty();
    let tag = exact_tag();

    let display_version =
    if !dirty && tag.as_deref() == Some(&expected_tag) {
        package_version
    } else {
        let hash = hash.as_deref().unwrap_or("unknown");
        if dirty {
            format!("{package_version} ({hash}-dirty)")
        } else {
            format!("{package_version} ({hash})")
        }
    };

    println!("cargo:rustc-env=MARGARINE_DISPLAY_VERSION={display_version}");
}
