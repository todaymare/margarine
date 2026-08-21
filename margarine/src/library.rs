use std::{
    fmt,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use sha2::{Digest, Sha256};
use toml_edit::{value, DocumentMut, Item, Table};

const REQUIRED_COMMANDS: &[&str] = &["clang", "git", "llvm-ar"];
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

#[derive(Debug, PartialEq, Eq)]
pub enum NativeBackend {
    C,
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
        "c" => NativeBackend::C,
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
    let tempdir = tempfile::tempdir()?;

    for &target in &manifest.targets {
        build_target(&manifest, target, tempdir.path())?;
    }

    create_share(path, &manifest, tempdir.path())?;

    let staged_path = tempdir.keep();

    let _ = std::fs::remove_dir_all(path.join("build"));
    if let Err(error) = std::fs::rename(&staged_path, path.join("build")) {
        let _ = std::fs::remove_dir_all(&staged_path);
        return Err(error);
    }

    Ok(())
}

fn create_generated_source(
    manifest: &LibraryManifest,
    output_dir: &Path,
    destination: &Path,
) -> io::Result<()> {
    let generated_source = destination.join("lib.mar");
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
    let package_path = share_path.join(&manifest.name);
    std::fs::create_dir_all(&package_path)?;

    create_generated_source(manifest, output_dir, &package_path)?;

    let library_path = project_path.join("lib");
    if library_path.is_dir() {
        copy_directory(&library_path, &package_path.join("lib"))?;
    }

    run_git(&package_path, &["init"])?;
    run_git(&package_path, &["add", "--all"])?;
    run_git_with_args(&package_path, &["commit", "-m", manifest.version.as_str()])
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
) -> io::Result<()> {
    let sources = collect_c_sources(&manifest.native_path)?;
    if sources.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "no C source files found in {}",
                manifest.native_path.display()
            ),
        ));
    }

    let target_name = target.margarine_target_triple();
    let object_dir = output_dir.join(".objects").join(&target_name);
    std::fs::create_dir_all(&object_dir)?;
    let c_target = target.c_target_triple();
    let mut objects = Vec::with_capacity(sources.len());
    for (index, source) in sources.iter().enumerate() {
        let object = object_dir.join(format!("{index}.o"));
        let output = Command::new("clang")
            .args(["-target", c_target.as_str(), "-c"])
            .arg(source)
            .arg("-o")
            .arg(&object)
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "clang failed for {}: {}",
                target_name,
                command_error(&output)
            )));
        }
        objects.push(object);
    }

    let target_dir = output_dir.join(&target_name);
    std::fs::create_dir(&target_dir)?;
    let artifact = target_dir.join(format!("{}.a", manifest.name));
    let mut archive = Command::new("llvm-ar");
    archive.arg("rcs").arg(&artifact).args(&objects);
    let output = archive.output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "{} failed for {}: {}",
            "llvm-ar",
            target_name,
            command_error(&output)
        )));
    }

    // The archive contains the object files, so they are no longer needed
    // once llvm-ar has completed successfully.
    std::fs::remove_dir_all(&object_dir)?;

    Ok(())
}

fn collect_c_sources(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            sources.extend(collect_c_sources(&path)?);
        } else if path.extension().is_some_and(|extension| extension == "c") {
            sources.push(path);
        }
    }
    Ok(sources)
}

fn command_error(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if stderr.is_empty() {
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    } else {
        stderr
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


    run_git(destination, &["init"])?;
    std::fs::write(destination.join(".gitignore"), "build")?;

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
        native.insert("backend".into(), "c".into());

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
        targets.insert("x86_64-unknown-linux-gnu".into(), Table::new().into());
        targets.insert("aarch64-unknown-linux-gnu".into(), Table::new().into());
        targets.insert("wasm32-unknown-unknown".into(), Table::new().into());
        toml["target"] = targets.into();

        toml["target"]
            .as_table_mut()
            .expect("target should be a table")
            .set_implicit(true);
    }

    std::fs::write(path.join("margarine.toml"), toml.to_string())?;

    std::fs::create_dir_all(path.join("native"))?;
    const DEFAULT_NATIVE: &str = "#include <stdint.h>\n\nuint64_t margarine_add(uint64_t left, uint64_t right) {\n    return left + right;\n}\n";
    std::fs::write(path.join("native/lib.c"), DEFAULT_NATIVE)?;

    const DEFAULT_STR: &str = "\
pub fn add(a: int, b: int): int {
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
