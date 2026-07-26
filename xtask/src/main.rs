use std::{env, fs::{self, File}, io, path::{Path, PathBuf}, process::{Command, ExitCode}};

use flate2::{write::GzEncoder, Compression};
use git2::Repository;
use margarine_installer::{executable_name, installer_name, static_library_name};
use tar::Builder;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() 
    else {
        eprintln!("usage: cargo run -p xtask -- bundle [--target <triple>] [--out <directory>] [--std-ref <ref>]");
        return ExitCode::from(2);
    };



    if command != "bundle" {
        eprintln!("unknown xtask command '{command}'");
        return ExitCode::from(2)
    }

    bundle(args.collect()).unwrap();
    ExitCode::SUCCESS
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
    let target = target.ok_or(invalid("specify a target"))?;
    let (_, rust_target) = margarine_installer::MAPPINGS.iter().find(|x| x.0 == target).unwrap();

    if !output.is_absolute() {
        output = root.join(output);
    }


    println!("creating bundle");

    let bundle = output.join(format!("margarine-{version}"));
    let target_bundle = bundle.join(&target);
    if target_bundle.exists() {
        fs::remove_dir_all(&target_bundle)?;
    }

    let mut cargo = Command::new("cargo");
    cargo.current_dir(&root)
        .args(["build", "--profile", "dist", "-p", "margarine", "-p", "runtime", "--target", rust_target]);

    run(&mut cargo, "building compiler and runtime")?;

    let release_dir = root.join("target").join(&rust_target).join("dist");

    let bin_dir = target_bundle.join("bin");
    let runtime_dir = target_bundle.join("lib");
    let share_dir = bundle.join("share");
    let std_dir = share_dir.join("std");
    let core_dir = share_dir.join("core");
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

    let bin = bin_dir.join(executable_name());
    let installer = bin_dir.join(installer_name());
    let static_lib = runtime_dir.join(static_library_name());

    copy_file(&release_dir.join(executable_name()), &bin)?;
    copy_file(&release_dir.join(static_library_name()), &static_lib)?;

    println!("creating installer payload");
    let tar_gz = tempfile::Builder::new()
        .prefix("margarine-")
        .tempfile()?;

    let encoder = GzEncoder::new(&tar_gz, Compression::none());
    let mut tar = Builder::new(encoder);

    tar.append_path_with_name(
        &bin,
        "margarine",
    )?;


    tar.append_path_with_name(
        &static_lib,
        "libmargarine_rt.a",
    )?;


    tar.append_dir_all(
        "share",
        share_dir,
    )?;

    tar.into_inner()?.finish()?;

    println!("building installer");

    let mut cargo = Command::new("cargo");
    cargo.current_dir(&root)
        .args(["build", "--profile", "dist", "-p", "margarine-installer", "--target", rust_target])
        .env("MARGARINE_INSTALL_VERSION", env!("CARGO_PKG_VERSION"))
        .env("MARGARINE_INSTALL_TARGET", &target)
        .env("MARGARINE_INSTALL_PAYLOAD", tar_gz.path().to_str().unwrap());

    run(&mut cargo, "building installer")?;

    copy_file(&release_dir.join(installer_name()), &installer)?;

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
