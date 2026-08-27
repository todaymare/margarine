use semver::Version;

pub fn package_version() -> Version {
    directory_version(crate::VERSION)
        .expect("CARGO_PKG_VERSION must be valid SemVer")
}

pub(crate) fn directory_version(name: &str) -> Result<Version, String> {
    let version =
        Version::parse(name)
            .map_err(|error| format!("invalid version `{name}`: {error}"))?;
    Ok(version)
}

pub fn release_tag_version(tag: &str) -> Result<Version, String> {
    directory_version(tag.strip_prefix('v').unwrap_or(tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_prerelease_and_build_metadata() {
        assert_eq!(
            directory_version("1.2.3-rc.1+build.7").unwrap(),
            Version::parse("1.2.3-rc.1+build.7").unwrap(),
        );
        assert_eq!(
            release_tag_version("v1.2.3-rc.1+build.7").unwrap(),
            Version::parse("1.2.3-rc.1+build.7").unwrap(),
        );
    }

    #[test]
    fn rejects_malformed_versions() {
        for version in ["1.2", "1.2.3.4", "1.2.3-rc..1", "1.2.3+"] {
            assert!(directory_version(version).is_err(), "{version}");
        }
    }
}
