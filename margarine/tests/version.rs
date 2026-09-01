use semver::Version;

use margarine::version::release_tag_version;

#[test]
fn release_tags_accept_prerelease_and_build_metadata() {
    assert_eq!(
        release_tag_version("v1.2.3-rc.1+build.7").unwrap(),
        Version::parse("1.2.3-rc.1+build.7").unwrap(),
    );
}

#[test]
fn release_tags_reject_malformed_versions() {
    for tag in [
        "v1.2",
        "v1.2.3.4",
        "v1.2.3-rc..1",
        "v1.2.3+",
    ] {
        assert!(release_tag_version(tag).is_err(), "{tag}");
    }
}

#[test]
fn package_version_is_valid_semver() {
    let version = margarine::version::package_version();
    assert_eq!(version.to_string(), margarine::VERSION);
}

