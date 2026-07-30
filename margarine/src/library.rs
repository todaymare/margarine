use std::{
    fmt,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use sha2::{Digest, Sha256};
use toml_edit::{value, Array, DocumentMut, Item, Table};

const REQUIRED_COMMANDS: &[&str] = &["cargo", "git", "rustc", "rustup"];
const DEFAULT_EXPORT_FORMAT: &str = "{base-url}/{version}/{arch}/{name}";

/// Describes the executables that are required to release a library.
#[derive(Debug, PartialEq, Eq)]
pub struct MissingPrerequisites {
    pub commands: Vec<String>,
}

impl fmt::Display for MissingPrerequisites {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "missing required command(s): {}",
            self.commands.join(", ")
        )
    }
}

impl std::error::Error for MissingPrerequisites {}

/// Checks that the tools used by the library release workflow are available.
pub fn validate_prerequisites() -> Result<(), MissingPrerequisites> {
    let missing = REQUIRED_COMMANDS
        .iter()
        .copied()
        .filter(|command| {
            Command::new(command)
                .arg("--version")
                .output()
                .map_or(true, |output| !output.status.success())
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(MissingPrerequisites { commands: missing })
    }
}

/// Verifies that `path` contains a valid Margarine project manifest.
pub fn validate_project<P: AsRef<Path>>(path: P) -> io::Result<()> {
    read_manifest(path).map(|_| ())
}

#[derive(Debug, PartialEq, Eq)]
pub enum NativeBackend {
    Rust,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LibraryManifest {
    pub name: String,
    pub version: String,
    pub native_path: PathBuf,
    pub native_backend: NativeBackend,
    pub targets: Vec<crate::CompilationTarget>,
    pub source_path: PathBuf,
    pub export_base_url: Option<String>,
    pub export_format: String,
}

fn read_manifest<P: AsRef<Path>>(path: P) -> io::Result<LibraryManifest> {
    let path = path.as_ref();
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            format!("not a directory: {}", path.display()),
        ));
    }

    let manifest_path = path.join("margarine.toml");
    let manifest = std::fs::read_to_string(&manifest_path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "not a margarine project: {} is missing",
                    manifest_path.display()
                ),
            )
        } else {
            error
        }
    })?;

    let document = DocumentMut::from_str(&manifest).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid Margarine manifest {}: {error}",
                manifest_path.display()
            ),
        )
    })?;

    let Some(package) = document.get("package").and_then(Item::as_table) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid margarine manifest: missing [package] section",
        ));
    };

    let name = required_string(package, "name", "[package]")?;
    let version = required_string(package, "version", "[package]")?;

    let Some(native) = document.get("native").and_then(Item::as_table) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid margarine manifest: missing [native] section",
        ));
    };

    let native_path = path.join(required_string(native, "path", "[native]")?);
    if !native_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("native path does not exist: {}", native_path.display()),
        ));
    }

    let native_backend = match required_string(native, "backend", "[native]")?.as_str() {
        "rust" => NativeBackend::Rust,
        backend => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported native backend: {backend}"),
            ));
        }
    };

    let Some(targets) = document.get("target").and_then(Item::as_table) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid margarine manifest: missing [target] section",
        ));
    };

    let targets = targets
        .iter()
        .map(|(target, _)| {
            crate::CompilationTarget::try_from(target)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
        })
        .collect::<io::Result<Vec<_>>>()?;
    let source_path = path.join("lib.mar");
    if !source_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("library source does not exist: {}", source_path.display()),
        ));
    }

    let export = document.get("export").and_then(Item::as_table);
    let export_base_url = export
        .map(|export| optional_string(export, "base-url", "[export]"))
        .transpose()?
        .flatten();
    let export_format = export
        .map(|export| required_string(export, "format", "[export]"))
        .transpose()?
        .unwrap_or_else(|| DEFAULT_EXPORT_FORMAT.to_owned());

    Ok(LibraryManifest {
        name,
        version,
        native_path,
        native_backend,
        targets,
        source_path,
        export_base_url,
        export_format,
    })
}

fn required_string(table: &Table, key: &str, section: &str) -> io::Result<String> {
    table
        .get(key)
        .and_then(Item::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid margarine manifest: {section} requires {key}"),
            )
        })
}

fn optional_string(table: &Table, key: &str, section: &str) -> io::Result<Option<String>> {
    table
        .get(key)
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid manifest: {section} requires {key} to be a string"),
                )
            })
        })
        .transpose()
}

pub fn build<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    let manifest = read_manifest(path)?;
    validate_rust_targets(&manifest.targets)?;

    let tempdir = tempfile::tempdir()?;
    let cargo_target_dir = manifest.native_path.join("target");

    for &target in &manifest.targets {
        build_target(&manifest, target, tempdir.path(), &cargo_target_dir)?;
    }

    create_generated_source(&manifest, tempdir.path())?;
    create_share(path, &manifest, tempdir.path())?;

    let staged_path = tempdir.keep();

    let _ = std::fs::remove_dir_all(path.join("build"));
    if let Err(error) = std::fs::rename(&staged_path, path.join("build")) {
        let _ = std::fs::remove_dir_all(&staged_path);
        return Err(error);
    }

    Ok(())
}

fn create_generated_source(manifest: &LibraryManifest, output_dir: &Path) -> io::Result<()> {
    let generated_source = output_dir.join("lib.mar");
    std::fs::copy(&manifest.source_path, &generated_source)?;

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(generated_source)?;
    writeln!(file)?;

    for &target in &manifest.targets {
        let artifact = output_dir
            .join(target.llvm_target_triple())
            .join(format!("{}.a", manifest.name));
        let bytes = std::fs::read(&artifact)?;
        let hash = Sha256::digest(bytes);
        let mut url = manifest
            .export_format
            .replace("{version}", &manifest.version)
            .replace("{arch}", &target.margarine_target_triple())
            .replace("{name}", &format!("{}.a", manifest.name));
        if let Some(base_url) = &manifest.export_base_url {
            url = url.replace("{base-url}", base_url.trim_end_matches('/'));
        }
        writeln!(
            file,
            "@cfg(env(\"MARGARINE_COMPILATION_TARGET\", \"{}\"))",
            target.margarine_target_triple()
        )?;
        writeln!(file, "@hash(\"{}\")", hex::encode(hash))?;
        writeln!(file, "extern \"{}\";", url)?;
    }

    Ok(())
}

fn create_share(
    project_path: &Path,
    manifest: &LibraryManifest,
    output_dir: &Path,
) -> io::Result<()> {
    let share_path = output_dir.join("share");
    std::fs::create_dir(&share_path)?;

    let native_lib = manifest.native_path.join("src/lib.rs");
    std::fs::copy(&native_lib, share_path.join("lib.rs")).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("could not copy {}: {error}", native_lib.display()),
        )
    })?;

    let library_path = project_path.join("lib");
    if library_path.is_dir() {
        copy_directory(&library_path, &share_path.join("lib"))?;
    }

    run_git(&share_path, &["init"])?;
    run_git(&share_path, &["add", "--all"])?;
    run_git_with_args(&share_path, &["commit", "-m", manifest.version.as_str()])
}

fn copy_directory(source: &Path, destination: &Path) -> io::Result<()> {
    std::fs::create_dir(destination)?;

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else {
            std::fs::copy(source_path, destination_path)?;
        }
    }

    Ok(())
}

fn run_git(directory: &Path, args: &[&str]) -> io::Result<()> {
    run_git_with_args(directory, args)
}

fn run_git_with_args(directory: &Path, args: &[&str]) -> io::Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(directory)
        .env("GIT_AUTHOR_NAME", "Margarine")
        .env("GIT_AUTHOR_EMAIL", "margarine@localhost")
        .env("GIT_COMMITTER_NAME", "Margarine")
        .env("GIT_COMMITTER_EMAIL", "margarine@localhost")
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "git {} failed: {}",
            args.join(" "),
            command_error(&output)
        )))
    }
}

fn build_target(
    manifest: &LibraryManifest,
    target: crate::CompilationTarget,
    output_dir: &Path,
    cargo_target_dir: &Path,
) -> io::Result<()> {
    let rust_target = target.rust_target_triple();
    let output = Command::new("cargo")
        .args(["build", "--target", rust_target.as_str()])
        .arg("--target-dir")
        .arg(cargo_target_dir)
        .current_dir(&manifest.native_path)
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "cargo build failed for {}: {}",
            target.llvm_target_triple(),
            command_error(&output)
        )));
    }

    let artifact = find_static_library(&cargo_target_dir.join(&rust_target).join("debug"))?;
    std::fs::create_dir(output_dir.join(target.llvm_target_triple()))?;
    std::fs::rename(
        artifact,
        output_dir
            .join(target.llvm_target_triple())
            .join(&format!("{}.a", manifest.name)),
    )
}

fn command_error(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if stderr.is_empty() {
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    } else {
        stderr
    }
}

fn find_static_library(directory: &Path) -> io::Result<PathBuf> {
    let mut artifacts = std::fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "a"))
        .collect::<Vec<_>>();

    match artifacts.len() {
        1 => Ok(artifacts.pop().expect("one static library was found")),
        0 => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "cargo did not produce a static library in {}",
                directory.display()
            ),
        )),
        _ => Err(io::Error::other(format!(
            "cargo produced multiple static libraries in {}",
            directory.display()
        ))),
    }
}

fn validate_rust_targets(targets: &[crate::CompilationTarget]) -> io::Result<()> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("could not query installed Rust targets: {error}"),
            )
        })?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "could not query installed Rust targets: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let installed_output = String::from_utf8_lossy(&output.stdout);
    let installed = installed_output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect::<std::collections::HashSet<_>>();

    let missing = targets
        .iter()
        .map(|target| target.rust_target_triple())
        .filter(|target| !installed.contains(target.as_str()))
        .collect::<Vec<_>>();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Rust targets are not installed: {}", missing.join(", ")),
        ))
    }
}

pub fn init<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let destination = path.as_ref();

    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "library path must have a valid file name",
            )
        })?;

    if destination.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("destination already exists: {}", destination.display()),
        ));
    }

    let parent = destination.parent().unwrap_or_else(|| Path::new("."));

    // Create it beside the destination so the final rename stays on one filesystem.
    let tempdir = tempfile::tempdir_in(parent)?;

    init_temp(tempdir.path(), name)?;

    // Keep the temporary directory, then atomically rename it.
    let staged_path = tempdir.keep();

    if let Err(error) = std::fs::rename(&staged_path, destination) {
        let _ = std::fs::remove_dir_all(&staged_path);
        return Err(error);
    }

    Ok(())
}

fn init_temp<P: AsRef<Path>>(path: P, name: &str) -> io::Result<()> {
    let path = path.as_ref();

    let mut toml = DocumentMut::new();

    {
        let mut package = Table::new();

        package.insert("name".into(), name.into());
        package.insert("version".into(), "0.1.0".into());
        toml["package"] = package.into();
    }

    {
        let mut native = Table::new();
        native.insert("path".into(), "native".into());
        native.insert("backend".into(), "rust".into());

        toml["native"] = native.into();
    }

    {
        let mut export = Table::new();
        export
            .decor_mut()
            .set_suffix("\n# base-url = \"https://example.com/my-library\"");
        export.insert("format".into(), value("{base-url}/{version}/{arch}/{name}"));
        toml["export"] = Item::Table(export);
    }

    {
        let mut targets = Table::new();
        targets.insert("arm64-apple-darwin".into(), Table::new().into());
        targets.insert("wasm32-unknown-unknown".into(), Table::new().into());

        toml["target"] = targets.into();

        toml["target"]
            .as_table_mut()
            .expect("target should be a table")
            .set_implicit(true);
    }

    std::fs::write(path.join("margarine.toml"), toml.to_string())?;

    {
        let mut toml = Table::new();
        toml["workspace"]["resolver"] = value("1");
        toml["workspace"]["members"] = value(Array::new());

        std::fs::write(path.join("Cargo.toml"), toml.to_string())?;
    }

    let output = Command::new("cargo")
        .arg("init")
        .arg("--lib")
        .arg("native")
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "cargo init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    std::fs::remove_file(path.join("Cargo.toml"))?;

    {
        let path = path.join("native").join("Cargo.toml");
        let cargo = std::fs::read_to_string(&path)?;
        let mut toml = toml_edit::DocumentMut::from_str(&cargo).unwrap();

        toml.remove("dependencies").unwrap();
        toml["workspace"] = Table::new().into();

        let mut lib = Table::new();

        let mut array = Array::new();
        array.push("staticlib");

        lib["crate-type"] = value(array);

        toml.insert("lib".into(), lib.into());

        toml["dependencies"] = Table::new().into();
        toml["workspace"] = Table::new().into();

        std::fs::write(path, toml.to_string())?;
    }

    const DEFAULT_STR: &str = "\
fn add(a: int, b: int): int {
    a + b
}


@test
fn test_add() {
    assert(add(5, 3) == 8, \"addition failed\");
}
";

    std::fs::write(path.join("lib.mar"), DEFAULT_STR)?;

    Ok(())
}
