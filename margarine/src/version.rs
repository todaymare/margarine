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

