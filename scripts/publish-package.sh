#!/bin/sh
# Publish a library package to the CDN as a git repository.
#
# Usage: scripts/publish-package.sh <package-dir> <package-name> <version>
#
# The CDN serves git repos over smart HTTP but accepts no `git push`; its only
# write path is a create-only HTTP PUT per file (Bearer auth). So this builds a
# bare repo snapshot of <package-dir> and uploads the bare repo's on-disk files
# (HEAD, refs/, objects/, info/) to:
#
#   <base>/margarine/<version>/<name>/
#
# which is the URL the compiler's `resolve_url("pkg:<name>")` clones.
#
# Environment:
#   MARGARINE_CDN_BASE  base URL (default https://cdn.daymare.net)
#   CDN_DEPLOY_KEY      Bearer token for the PUT API

set -eu

PKG_DIR=$(CDPATH= cd -- "$1" && pwd)
PKG_NAME=$2
VERSION=$3
BASE=${MARGARINE_CDN_BASE:-https://cdn.daymare.net}
TOKEN=${CDN_DEPLOY_KEY:?set CDN_DEPLOY_KEY to the CDN Bearer token}
REMOTE_DIR="margarine/$VERSION/$PKG_NAME"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

WORK="$TMP/work"
BARE="$TMP/$PKG_NAME.git"

mkdir -p "$WORK"
git -C "$WORK" init -q
cp -R "$PKG_DIR"/. "$WORK"/
printf 'build/\nartifacts/\nbuild.lock\n.DS_Store\n' > "$WORK/.gitignore"
git -C "$WORK" add -A
git -C "$WORK" \
    -c user.email=release@margarine \
    -c user.name=margarine \
    commit -q -m "release $VERSION"

git clone -q --bare "$WORK" "$BARE"
git --git-dir="$BARE" gc -q --prune=now
git --git-dir="$BARE" update-server-info

for rel in $(cd "$BARE" && find . -type f | sed 's|^\./||'); do
    curl -fsS -X PUT \
        -H "Authorization: Bearer $TOKEN" \
        --data-binary "@$BARE/$rel" \
        "$BASE/$REMOTE_DIR/$rel" >/dev/null \
        || { echo "error: PUT $rel failed" >&2; exit 1; }
done

printf '%s\n' "published $PKG_NAME $VERSION -> $BASE/$REMOTE_DIR"
