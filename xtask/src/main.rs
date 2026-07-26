use std::{env, fs, io, path::{Path, PathBuf}, process::{Command, ExitCode}};

use git2::Repository;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("usage: cargo run -p xtask -- bundle [--target <triple>] [--out <directory>] [--std-ref <ref>]");
        return ExitCode::from(2);
    };

    match command.as_str() {
        "bundle" => match bundle(args.collect()) {
            Ok(path) => {
                println!("created {}", path.display());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("bundle failed: {error}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("unknown xtask command '{command}'");
            ExitCode::from(2)
        }
    }
}

fn bundle(args: Vec<String>) -> io::Result<PathBuf> {
    let mut target = None;
    let mut output = PathBuf::from("dist");
    let mut std_ref = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--target" => {
                index += 1;
                target = Some(args.get(index).ok_or_else(|| invalid("--target requires a value"))?.clone());
            }
            "--out" => {
                index += 1;
                output = PathBuf::from(args.get(index).ok_or_else(|| invalid("--out requires a value"))?);
            }
            "--std-ref" => {
                index += 1;
                std_ref = Some(args.get(index).ok_or_else(|| invalid("--std-ref requires a value"))?.clone());
            }
            value => return Err(invalid(&format!("unknown option '{value}'"))),
        }
        index += 1;
    }

    let root = workspace_root()?;
    let version = compiler_version(&root)?;
    let target = target.unwrap_or(host_target()?);
    let explicit_target = args.iter().any(|arg| arg == "--target");
    if !output.is_absolute() {
        output = root.join(output);
    }

    let bundle = output.join(format!("margarine-{version}"));
    let target_bundle = bundle.join(&target);
    if target_bundle.exists() {
        fs::remove_dir_all(&target_bundle)?;
    }

    let mut cargo = Command::new("cargo");
    cargo.current_dir(&root)
        .args(["build", "--profile", "dist", "-p", "margarine", "-p", "runtime"]);
    if explicit_target {
        cargo.args(["--target", &target]);
    }
    run(&mut cargo, "building compiler and runtime")?;

    let release_dir = if explicit_target {
        root.join("target").join(&target).join("dist")
    } else {
        root.join("target").join("dist")
    };

    let bin_dir = target_bundle.join("bin");
    let runtime_dir = target_bundle.join("lib");
    let std_dir = bundle.join("share/std");
    let core_dir = bundle.join("share/core");
    if std_dir.exists() {
        fs::remove_dir_all(&std_dir)?;
    }
    if core_dir.exists() {
        fs::remove_dir_all(&core_dir)?;
    }
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&runtime_dir)?;
    fs::create_dir_all(&core_dir)?;
    let std_source = checkout_standard_library(&root, &version, std_ref.as_deref())?;
    copy_dir(&std_source, &std_dir)?;

    copy_file(&release_dir.join(executable_name()), &bin_dir.join(executable_name()))?;
    copy_file(&release_dir.join(static_library_name()), &runtime_dir.join(static_library_name()))?;

    Ok(bundle)
}

fn compiler_version(root: &Path) -> io::Result<String> {
    let manifest = fs::read_to_string(root.join("margarine/Cargo.toml"))?;
    manifest
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            if key.trim() != "version" {
                return None;
            }
            Some(value.trim().trim_matches('"').to_string())
        })
        .ok_or_else(|| invalid("margarine/Cargo.toml has no package version"))
}

fn checkout_standard_library(root: &Path, version: &str, requested_ref: Option<&str>) -> io::Result<PathBuf> {
    let base = env::var("MARGARINE_DEFAULT_URL")
        .unwrap_or_else(|_| "https://pkg.daymare.net/margarine".to_string());
    let repository_url = format!("{}/std", base.trim_end_matches('/'));
    let checkout = root.join("target/xtask/std").join(version);

    if checkout.exists() {
        fs::remove_dir_all(&checkout)?;
    }
    if let Some(parent) = checkout.parent() {
        fs::create_dir_all(parent)?;
    }

    let repository = Repository::clone(&repository_url, &checkout)
        .map_err(|error| invalid(&format!("failed to clone std from {repository_url}: {error}")))?;

    let version_refs = requested_ref
        .map(|reference| vec![reference.to_string()])
        .unwrap_or_else(|| vec![format!("refs/tags/v{version}"), format!("refs/tags/{version}")]);
    let Some(object) = version_refs
        .iter()
        .find_map(|reference| repository.revparse_single(reference).ok())
    else {
        return Err(match requested_ref {
            Some(reference) => invalid(&format!("std repository has no ref {reference}")),
            None => invalid(&format!("std repository has no tag v{version} or {version}")),
        });
    };

    repository
        .checkout_tree(&object, None)
        .map_err(|error| invalid(&format!("failed to checkout std {version}: {error}")))?;
    repository
        .set_head_detached(object.id())
        .map_err(|error| invalid(&format!("failed to detach std at {version}: {error}")))?;

    Ok(checkout)
}

fn workspace_root() -> io::Result<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| invalid("xtask is not inside a workspace"))
}

fn host_target() -> io::Result<String> {
    let output = Command::new("rustc").arg("-vV").output()?;
    if !output.status.success() {
        return Err(invalid("rustc -vV failed"));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or_else(|| invalid("rustc -vV did not report a host target"))
}

fn executable_name() -> &'static str {
    if cfg!(windows) { "margarine.exe" } else { "margarine" }
}

fn static_library_name() -> &'static str {
    if cfg!(windows) { "margarine.lib" } else { "libmargarine.a" }
}

fn copy_file(source: &Path, destination: &Path) -> io::Result<()> {
    if !source.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("missing build output {}", source.display()),
        ));
    }
    fs::copy(source, destination)?;
    Ok(())
}

fn copy_dir(source: &Path, destination: &Path) -> io::Result<()> {
    if !source.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("missing package source {}", source.display()),
        ));
    }

    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &destination_path)?;
        } else {
            copy_file(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn run(command: &mut Command, description: &str) -> io::Result<()> {
    println!("{description}...");
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, format!("{description} exited with {status}")))
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
