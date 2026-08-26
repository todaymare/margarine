use std::{io, path::PathBuf};

use colourful::ColourBrush;
use margarine::progress::{item_progress, StatusLine};

/// Exclusive cross-process lock over `artifacts/`, held for the lifetime of
/// the returned guard. The lock file lives outside the deleted tree
/// (`artifacts.lock` next to `build.lock`) because removing a lock file while
/// held would leave latecomers locking a fresh inode with no mutual exclusion.
pub(super) struct ArtifactsLock {
    file: std::fs::File,
}

impl ArtifactsLock {
    pub(super) fn acquire() -> Self {
        use fs2::FileExt;

        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open("artifacts.lock")
            .unwrap_or_else(|error| panic!("cannot open artifacts.lock: {error}"));

        match file.try_lock_exclusive() {
            Ok(()) => {},
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                let waiting = StatusLine::start("Waiting for artifacts lock");
                file.lock_exclusive()
                    .unwrap_or_else(|error| panic!("cannot lock artifacts: {error}"));
                waiting.clear();
            },
            Err(error) => panic!("cannot lock artifacts: {error}"),
        }

        ArtifactsLock { file }
    }
}

impl Drop for ArtifactsLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}


pub(super) fn clean(cache: &str) {
    let Ok(metadata) = std::fs::symlink_metadata(cache) else {
        println!("{}", "nothing to clean".dim());
        return;
    };
    let cache_is_dir = metadata.file_type().is_dir();

    let mut paths: Vec<PathBuf> = Vec::new();
    if cache_is_dir {
        let mut stack = vec![PathBuf::from(cache)];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.flatten() {
                let path = entry.path();
                if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                    stack.push(path);
                } else {
                    paths.push(path);
                }
            }
        }
    } else {
        paths.push(PathBuf::from(cache));
    }

    let total_bytes: u64 = paths.iter()
        .filter_map(|path| std::fs::symlink_metadata(path).ok())
        .map(|meta| meta.len())
        .sum();
    let total = format_bytes(total_bytes);
    let progress =
    item_progress(
        paths.len() as u64,
        format!("Removing {total}"),
    );

    let mut removed = 0usize;
    let mut failures = 0usize;
    for path in &paths {
        match std::fs::remove_file(path) {
            Ok(()) => removed += 1,
            Err(_) => {
                failures += 1;
                progress.suspend(|| {
                    eprintln!("{} could not remove {}", "warning:".yellow().bold(), path.display());
                });
            }
        }
        progress.inc(1);
    }

    if cache_is_dir {
        if let Err(error) = std::fs::remove_dir_all(cache) {
            progress.suspend(|| {
                eprintln!("{} could not remove {cache}: {error}", "warning:".yellow().bold());
            });
            failures += 1;
        }
    }
    progress.finish();

    if failures == 0 {
        println!("{} Removed {} files, {}",
            "✓".green(),
            removed,
            total.cyan(),
        );
    } else {
        println!("{} Removed {} files, {} failed; use `--update` to reset the cache",
            "!".yellow().bold(),
            removed,
            failures,
        );
    }
}


/// Resolves the effective cache directory, resetting the build cache first
/// when `update` is set. The lock file lives at the project root and is
/// cache-independent; the commit map it holds is only consulted against the
/// fresh clones a reset triggers anyway.
pub(super) fn reset_cache_if(update: bool, cache: Option<String>) -> String {
    let cache = cache.unwrap_or_else(|| "artifacts".to_string());
    if update {
        if std::fs::exists("build.lock").unwrap() {
            std::fs::remove_file("build.lock").unwrap();
        }

        clean(&cache);
    }

    cache
}

pub(super) fn format_bytes(bytes: u64) -> String {
    let units = ["b", "KiB", "MiB", "GiB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < units.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", units[unit])
    }
}


#[cfg(test)]
mod tests {
    use super::clean;

    #[test]
    fn clean_does_not_follow_directory_symlinks() {
        let cache = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("keep");
        std::fs::write(&outside_file, "keep").unwrap();
        std::os::unix::fs::symlink(outside.path(), cache.path().join("outside")).unwrap();

        clean(cache.path().to_str().unwrap());

        assert!(outside_file.is_file());
        assert!(!cache.path().exists());
    }


    #[test]
    fn clean_removes_a_cache_symlink_without_following_it() {
        let parent = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("keep");
        let cache = parent.path().join("cache");
        std::fs::write(&outside_file, "keep").unwrap();
        std::os::unix::fs::symlink(outside.path(), &cache).unwrap();

        clean(cache.to_str().unwrap());

        assert!(outside_file.is_file());
        assert!(!cache.exists());
        assert!(std::fs::symlink_metadata(cache).is_err());
    }


    #[test]
    fn clean_removes_a_file_used_as_the_cache_path() {
        let parent = tempfile::tempdir().unwrap();
        let cache = parent.path().join("cache");
        std::fs::write(&cache, "stale cache").unwrap();

        clean(cache.to_str().unwrap());

        assert!(!cache.exists());
    }
}
