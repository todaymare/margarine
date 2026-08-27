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
command -v awk >/dev/null 2>&1 || fail "awk is required"
[ -n "${HOME:-}" ] || fail "HOME is required"

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

root="$HOME/.margarine"
active="$root/bin/margarine"
mkdir -p "$root"
lock="$root/install.lock"
mkdir "$lock" 2>/dev/null || fail "another installation is in progress"

temporary_dir=
published_version=
expected_link=
complete=0
cleanup() {
    if [ "$complete" -eq 0 ]; then
        if [ -n "$expected_link" ] && [ -L "$active" ] \
            && [ "$(readlink "$active")" = "$expected_link" ]; then
            rm -f "$active"
        fi
        if [ -n "$published_version" ]; then
            rm -rf "$published_version"
        fi
    fi
    if [ -n "$temporary_dir" ]; then
        rm -rf "$temporary_dir"
    fi
    rmdir "$lock" 2>/dev/null || true
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

if [ -e "$active" ] || [ -L "$active" ]; then
    fail "margarine is already installed at $active; run 'margarine update' instead"
fi

temporary_dir=$(mktemp -d "$root/.install.XXXXXX")

download() {
    curl --fail --silent --show-error --location --retry 3 \
        --user-agent margarine-install.sh --output "$2" "$1"
}

download_verified() {
    asset_name=$1
    checksum_name="$asset_name.sha256"
    archive_path="$temporary_dir/$asset_name"
    checksum_path="$temporary_dir/$checksum_name"

    download "$RELEASE_DOWNLOAD_URL/$checksum_name" "$checksum_path"
    expected_checksum=$(awk 'NF { print $1; exit }' "$checksum_path")
    case "$expected_checksum" in
        ''|*[!0-9A-Fa-f]*)
            fail "invalid checksum in $checksum_name"
            ;;
    esac
    [ "${#expected_checksum}" -eq 64 ] || fail "invalid checksum in $checksum_name"

    download "$RELEASE_DOWNLOAD_URL/$asset_name" "$archive_path"
    if command -v sha256sum >/dev/null 2>&1; then
        actual_checksum=$(sha256sum "$archive_path" | awk '{ print $1 }')
    elif command -v shasum >/dev/null 2>&1; then
        actual_checksum=$(shasum -a 256 "$archive_path" | awk '{ print $1 }')
    else
        fail "sha256sum or shasum is required"
    fi

    expected_checksum=$(printf '%s' "$expected_checksum" | tr '[:upper:]' '[:lower:]')
    actual_checksum=$(printf '%s' "$actual_checksum" | tr '[:upper:]' '[:lower:]')
    [ "$expected_checksum" = "$actual_checksum" ] \
        || fail "checksum mismatch for $asset_name"
}

validate_archive() {
    archive_path=$1
    required_prefix=$2
    entries_path="$temporary_dir/entries"
    verbose_path="$temporary_dir/entries.verbose"

    tar -tzf "$archive_path" > "$entries_path" \
        || fail "cannot list release archive $(basename "$archive_path")"
    awk -v required_prefix="$required_prefix" '
        {
            path = $0
            sub(/\/$/, "", path)
            if (path == "" || path ~ /^\// || path ~ /[^A-Za-z0-9._+\/-]/) {
                exit 1
            }
            count = split(path, parts, "/")
            for (component = 1; component <= count; component++) {
                if (parts[component] == "" || parts[component] == "." || parts[component] == "..") {
                    exit 1
                }
            }
            if (required_prefix != "" && path != required_prefix &&
                index(path, required_prefix "/") != 1) {
                exit 1
            }
        }
        END {
            if (NR == 0) {
                exit 1
            }
        }
    ' "$entries_path" || fail "release archive contains an unsafe path"

    tar -tvzf "$archive_path" > "$verbose_path" \
        || fail "cannot inspect release archive $(basename "$archive_path")"
    awk '
        substr($0, 1, 1) != "-" && substr($0, 1, 1) != "d" {
            exit 1
        }
    ' "$verbose_path" || fail "release archive contains a link or special file"
}

compiler_archive="margarine-$target.tar.gz"
toolchain_archive="margarine-toolchain-$target.tar.gz"
printf '%s\n' "Downloading margarine for $target"
download_verified "$compiler_archive"
download_verified "$toolchain_archive"
validate_archive "$temporary_dir/$compiler_archive" ""
validate_archive "$temporary_dir/$toolchain_archive" "libs"

staged_version="$temporary_dir/version"
staged_bin="$staged_version/bin"
staged_toolchain="$staged_version/toolchains/$target"
mkdir -p "$staged_bin" "$staged_toolchain"
tar -xzf "$temporary_dir/$compiler_archive" -C "$staged_bin"
tar -xzf "$temporary_dir/$toolchain_archive" -C "$staged_toolchain"

binary="$staged_bin/margarine"
[ -f "$binary" ] || fail "compiler archive does not contain margarine"
chmod +x "$binary"

has_runtime=false
for runtime in "$staged_toolchain"/libs/*; do
    if [ -f "$runtime" ]; then
        has_runtime=true
        break
    fi
done
[ "$has_runtime" = true ] || fail "toolchain archive does not contain runtime libraries"

version_output=$("$binary" --version) \
    || fail "downloaded compiler failed its version check"
case "$version_output" in
    "margarine "*)
        version=${version_output#margarine }
        ;;
    *)
        fail "downloaded compiler returned an invalid version"
        ;;
esac
case "$version" in
    ''|*[!0-9A-Za-z.+-]*)
        fail "downloaded compiler returned an invalid version"
        ;;
esac
[ "$version_output" = "margarine $version" ] \
    || fail "downloaded compiler returned an invalid version"

version_dir="$root/$version"
if [ -e "$version_dir" ] || [ -L "$version_dir" ]; then
    fail "refusing to overwrite existing installation $version_dir"
fi

mv "$staged_version" "$version_dir"
published_version=$version_dir

mkdir -p "$root/bin"
if [ -e "$active" ] || [ -L "$active" ]; then
    fail "margarine was installed by another process"
fi
expected_link="../$version/bin/margarine"
pending="$root/bin/.margarine-link.$$"
ln -s "$expected_link" "$pending"
mv "$pending" "$active"

"$active" --version >/dev/null \
    || fail "installed compiler failed its final check"

complete=1
printf '%s\n' "Installed margarine $version at $version_dir"
case ":${PATH:-}:" in
    *":$root/bin:"*) ;;
    *)
        printf '%s\n' "Add margarine to PATH:"
        printf '  export PATH="%s/bin:$PATH"\n' "$root"
        ;;
esac
