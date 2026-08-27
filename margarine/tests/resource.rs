use std::path::{Path, PathBuf};

use margarine::resource::installation_from_executable;
use semver::Version;

#[test]
fn managed_installation_accepts_semver_metadata() {
    for (path, version) in [
        (Path::new("/opt/margarine/0.1.0/bin/margarine"), "0.1.0"),
        (
            Path::new("/opt/margarine/0.1.0-rc.1/bin/margarine"),
            "0.1.0-rc.1",
        ),
        (
            Path::new("/opt/margarine/0.1.0+build.7/bin/margarine"),
            "0.1.0+build.7",
        ),
    ] {
        assert_eq!(
            installation_from_executable(path).unwrap(),
            (
                PathBuf::from("/opt/margarine"),
                Version::parse(version).unwrap(),
            ),
        );
    }

    for unmanaged in [
        Path::new("/opt/margarine"),
        Path::new("/opt/margarine/bin/margarine"),
        Path::new("/opt/margarine/0.1.0/bin/not-margarine"),
        Path::new("/opt/margarine/1.2.3.4/bin/margarine"),
    ] {
        let error = installation_from_executable(unmanaged).unwrap_err();
        assert!(error.contains("not managed by the self-updater"));
        assert!(error.contains(&unmanaged.display().to_string()));
    }
}
