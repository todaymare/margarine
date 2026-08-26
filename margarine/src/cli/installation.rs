use std::{fs, os::unix::fs as unix_fs, path::{Path, PathBuf}};

use colourful::ColourBrush;
use fs2::FileExt as _;
use margarine::{progress::StatusLine, CompilationTarget};

use super::{
    distribution::{download_checked_assets, extract_archive, run_binary_check, Asset, Release},
    toolchain,
    TICK_GLYPH,
};


pub(super) enum CompilerSource<'a> {
    Current(&'a Path),
    Release {
        archive: &'a Asset,
        checksum: &'a Asset,
    },
}


pub(super) struct Installation {
    root: PathBuf,
    _lock: fs::File,
}

impl Installation {
    pub(super) fn acquire(root: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&root)
            .map_err(|error| format!("cannot create installation root {}: {error}", root.display()))?;
        let lock =
            fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(root.join("update.lock"))
                .map_err(|error| format!("cannot open installation lock: {error}"))?;
        match lock.try_lock_exclusive() {
            Ok(()) => {},
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                let waiting = StatusLine::start("Waiting for installation lock");
                lock.lock_exclusive()
                    .map_err(|error| format!("cannot acquire installation lock: {error}"))?;
                waiting.clear();
            },
            Err(error) =>
                return Err(format!("cannot acquire installation lock: {error}")),
        }
        Ok(Self { root, _lock: lock })
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn version_path(&self, version: &str) -> PathBuf {
        self.root.join(version)
    }

    pub(super) fn ensure_version_absent(&self, version: &str) -> Result<(), String> {
        let destination = self.version_path(version);
        match fs::symlink_metadata(&destination) {
            Ok(_) => Err(format!(
                "refusing to overwrite existing installation {}",
                destination.display(),
            )),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "cannot inspect installation target {}: {error}",
                destination.display(),
            )),
        }
    }

    pub(super) fn install_release(
        &self,
        version: &str,
        release: &Release,
        compiler: CompilerSource<'_>,
        targets: &[CompilationTarget],
    ) -> Result<(), String> {
        let release_version =
            release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);
        if release_version != version {
            return Err(format!(
                "release metadata is for {}, but version {version} was requested",
                release.tag_name,
            ));
        }
        self.ensure_version_absent(version)?;
        let previous_active = self.active_target()?;

        for target in targets {
            toolchain::checked_toolchain_assets(release, *target)?;
        }

        let staging =
            tempfile::Builder::new()
                .prefix(".staging-")
                .tempdir_in(&self.root)
                .map_err(|error| format!("cannot create installation staging directory: {error}"))?;
        let staged_version = staging.path();
        let staged_bin = staged_version.join("bin");
        fs::create_dir_all(&staged_bin)
            .map_err(|error| format!("cannot create staged binary directory: {error}"))?;
        let staged_executable = staged_bin.join("margarine");

        match compiler {
            CompilerSource::Current(source) => {
                fs::copy(source, &staged_executable)
                    .map_err(|error| format!("cannot copy current executable: {error}"))?;
            }
            CompilerSource::Release { archive, checksum } => {
                let mut archive_file = download_checked_assets(archive, checksum)?;
                println!(
                    "{} Downloaded compiler archive ({})",
                    TICK_GLYPH.green().bold(),
                    super::artifacts::format_bytes(archive.size),
                );
                extract_archive(archive_file.as_file_mut(), &staged_bin)?;
            }
        }

        let checking = StatusLine::start("Verifying binary (1/2)");
        if let Err(error) = run_binary_check(&staged_executable) {
            checking.clear();
            return Err(format!("invalid compiler executable: {error}"));
        }
        checking.clear();
        println!("{} Verified binary (1/2)", TICK_GLYPH.green().bold());

        for target in targets {
            toolchain::install(&staged_version, *target, release)?;
        }

        self.ensure_version_absent(version)?;
        let staged_version = staging.keep();
        let installed_version = self.version_path(version);
        if let Err(error) = fs::rename(&staged_version, &installed_version) {
            let _ = fs::remove_dir_all(&staged_version);
            return Err(format!("cannot publish version {version}: {error}"));
        }

        let active_before_activation =
        match self.active_target() {
            Ok(target) => target,
            Err(error) => {
                let _ = fs::remove_dir_all(&installed_version);
                return Err(error);
            },
        };
        if active_before_activation != previous_active {
            let _ = fs::remove_dir_all(&installed_version);
            return Err(
                "the active installation changed while the new version was being staged; \
                 the new version was not activated"
                    .into(),
            );
        }

        if let Err(error) = self.replace_active_target(version) {
            let _ = fs::remove_dir_all(&installed_version);
            return Err(error);
        }

        let active_executable = self.root.join("bin/margarine");
        let checking = StatusLine::start("Verifying binary (2/2)");
        if let Err(error) = run_binary_check(&active_executable) {
            checking.clear();
            let restore =
                self.restore_active_target(previous_active.as_deref(), version);
            if restore.is_ok() {
                let _ = fs::remove_dir_all(&installed_version);
                return Err(format!(
                    "installed binary failed its final check: {error}; the previous installation was restored",
                ));
            }
            return Err(format!(
                "installed binary failed its final check: {error}; rollback also failed: {}. Version {version} remains installed",
                restore.unwrap_err(),
            ));
        }
        checking.clear();
        println!("{} Verified binary (2/2)", TICK_GLYPH.green().bold());
        Ok(())
    }

    fn active_target(&self) -> Result<Option<PathBuf>, String> {
        let active = self.root.join("bin/margarine");
        match fs::symlink_metadata(&active) {
            Ok(metadata) if metadata.file_type().is_symlink() =>
                fs::read_link(&active)
                    .map(Some)
                    .map_err(|error| format!("cannot read active executable link: {error}")),
            Ok(_) => Err(format!(
                "refusing to replace active executable {} because it is not a symbolic link",
                active.display(),
            )),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("cannot inspect active executable: {error}")),
        }
    }

    fn replace_active_target(&self, version: &str) -> Result<(), String> {
        let bin_dir = self.root.join("bin");
        fs::create_dir_all(&bin_dir)
            .map_err(|error| format!("cannot create active binary directory: {error}"))?;
        let pending = bin_dir.join(".margarine-link");
        match fs::remove_file(&pending) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("cannot remove stale activation link: {error}")),
        }
        unix_fs::symlink(format!("../{version}/bin/margarine"), &pending)
            .map_err(|error| format!("cannot create activation link: {error}"))?;
        if let Err(error) = fs::rename(&pending, bin_dir.join("margarine")) {
            let _ = fs::remove_file(&pending);
            return Err(format!("cannot activate version {version}: {error}"));
        }
        Ok(())
    }

    fn restore_active_target(
        &self,
        previous: Option<&Path>,
        failed_version: &str,
    ) -> Result<(), String> {
        let active = self.root.join("bin/margarine");
        let expected = PathBuf::from(format!("../{failed_version}/bin/margarine"));
        if self.active_target()?.as_deref() != Some(expected.as_path()) {
            return Err(
                "the active installation changed after activation; refusing to overwrite it"
                    .into(),
            );
        }
        let Some(previous) = previous else {
            return fs::remove_file(&active)
                .map_err(|error| format!("cannot remove failed activation: {error}"));
        };
        let pending = self.root.join("bin/.margarine-link");
        match fs::remove_file(&pending) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("cannot remove stale rollback link: {error}")),
        }
        unix_fs::symlink(previous, &pending)
            .map_err(|error| format!("cannot create rollback link: {error}"))?;
        if let Err(error) = fs::rename(&pending, &active) {
            let _ = fs::remove_file(&pending);
            return Err(format!("cannot restore previous installation: {error}"));
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt as _;

    use super::*;

    fn write_executable(path: &Path, script: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn installation_lock_serializes_transactions() {
        let dir = tempfile::tempdir().unwrap();
        let installation = Installation::acquire(dir.path().to_path_buf()).unwrap();
        let lock_path = dir.path().join("update.lock");
        let contender =
            fs::OpenOptions::new()
                .write(true)
                .open(&lock_path)
                .unwrap();

        // flock is per open-file-description: a second descriptor from this
        // process must conflict while the installation holds the lock.
        assert!(contender.try_lock_exclusive().is_err());
        drop(installation);
        // Release is asynchronous on some platforms; poll instead of assuming
        // the very next try succeeds.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match contender.try_lock_exclusive() {
                Ok(()) => break,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {},
                Err(error) => panic!("unexpected lock error: {error}"),
            }
            assert!(
                std::time::Instant::now() < deadline,
                "installation lock was not released after drop",
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn activation_switches_and_restores_the_stable_relative_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let installation = Installation::acquire(dir.path().to_path_buf()).unwrap();
        let old = dir.path().join("0.1.0/bin/margarine");
        let new = dir.path().join("0.2.0/bin/margarine");
        write_executable(&old, "echo old");
        write_executable(&new, "echo new");
        fs::create_dir_all(dir.path().join("bin")).unwrap();
        unix_fs::symlink("../0.1.0/bin/margarine", dir.path().join("bin/margarine"))
            .unwrap();

        let previous = installation.active_target().unwrap();
        installation.replace_active_target("0.2.0").unwrap();
        assert_eq!(
            fs::read_link(dir.path().join("bin/margarine")).unwrap(),
            Path::new("../0.2.0/bin/margarine"),
        );

        installation
            .restore_active_target(previous.as_deref(), "0.2.0")
            .unwrap();
        assert_eq!(
            fs::read_link(dir.path().join("bin/margarine")).unwrap(),
            Path::new("../0.1.0/bin/margarine"),
        );
    }

    #[test]
    fn installation_refuses_non_directory_destinations() {
        let dir = tempfile::tempdir().unwrap();
        let installation = Installation::acquire(dir.path().to_path_buf()).unwrap();
        fs::write(dir.path().join("0.2.0"), "occupied").unwrap();

        let error = installation.ensure_version_absent("0.2.0").unwrap_err();

        assert!(error.contains("refusing to overwrite existing installation"));
    }
}
