#!/bin/sh
set -eu

REPOSITORY=${MARGARINE_GITHUB_REPOSITORY:-todaymare/margarine}
RELEASE_DOWNLOAD_URL=${MARGARINE_RELEASE_DOWNLOAD_URL:-https://github.com/$REPOSITORY/releases/latest/download}

fail() {
    printf '%s\n' "error: $*" >&2
    exit 1
}

command -v curl >/dev/null 2>&1 || fail "curl is required"
command -v tar >/dev/null 2>&1 || fail "tar is required"

os=$(uname -s)
arch=$(uname -m)
case "$os:$arch" in
    Darwin:arm64|Darwin:aarch64)
        target=arm64-apple-darwin
        ;;
    Linux:x86_64|Linux:amd64)
        target=x86_64-unknown-linux-gnu
        ;;
    Linux:aarch64|Linux:arm64)
        target=aarch64-unknown-linux-gnu
        ;;
    *)
        fail "unsupported platform: $os $arch"
        ;;
esac

archive_name="margarine-$target.tar.gz"
checksum_name="$archive_name.sha256"
temporary_dir=$(mktemp -d "${TMPDIR:-/tmp}/margarine-install.XXXXXX")

cleanup() {
    rm -rf "$temporary_dir"
}
trap cleanup EXIT HUP INT TERM

download() {
    curl --fail --silent --show-error --location --retry 3 \
        --user-agent margarine-install.sh --output "$2" "$1"
}

printf '%s\n' "Downloading margarine for $target"
download "$RELEASE_DOWNLOAD_URL/$checksum_name" "$temporary_dir/$checksum_name"
expected_checksum=$(awk 'NF { print $1; exit }' "$temporary_dir/$checksum_name")
case "$expected_checksum" in
    ''|*[!0-9A-Fa-f]*)
        fail "invalid checksum in $checksum_name"
        ;;
esac
[ "${#expected_checksum}" -eq 64 ] || fail "invalid checksum in $checksum_name"

download "$RELEASE_DOWNLOAD_URL/$archive_name" "$temporary_dir/$archive_name"
if command -v sha256sum >/dev/null 2>&1; then
    actual_checksum=$(sha256sum "$temporary_dir/$archive_name" | awk '{ print $1 }')
elif command -v shasum >/dev/null 2>&1; then
    actual_checksum=$(shasum -a 256 "$temporary_dir/$archive_name" | awk '{ print $1 }')
else
    fail "sha256sum or shasum is required"
fi

expected_checksum=$(printf '%s' "$expected_checksum" | tr '[:upper:]' '[:lower:]')
actual_checksum=$(printf '%s' "$actual_checksum" | tr '[:upper:]' '[:lower:]')
[ "$expected_checksum" = "$actual_checksum" ] \
    || fail "checksum mismatch for $archive_name"

mkdir "$temporary_dir/extracted"
tar -xzf "$temporary_dir/$archive_name" -C "$temporary_dir/extracted"
binary="$temporary_dir/extracted/margarine"
[ -f "$binary" ] || fail "release archive does not contain margarine"
chmod +x "$binary"

if [ -z "${MARGARINE_RELEASES_API+x}" ]; then
    export MARGARINE_RELEASES_API="https://api.github.com/repos/$REPOSITORY/releases"
fi

"$binary" install --yes
