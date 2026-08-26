#!/bin/sh
# Build the margarine native runtime archives (core + std) for one
# compilation target.
#
# Usage: scripts/build-toolchains.sh <target-triple> <output-dir>
#
# Compiles core/native/lib.c and std/native/lib.c with clang for the given
# target triple and writes libcore.a + libstd.a into <output-dir>.
#
# The library source root is resolved like the compiler does: an explicit
# MARGARINE_LIBRARY_DIR wins, then <repo>/libraries, then the sibling
# <repo>/../libraries checkout.

set -eu

TARGET=${1:?usage: build-toolchains.sh <target-triple> <output-dir>}
OUT=${2:?usage: build-toolchains.sh <target-triple> <output-dir>}

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

if [ -n "${MARGARINE_LIBRARY_DIR:-}" ]; then
    LIB_DIR=$MARGARINE_LIBRARY_DIR
elif [ -d "$REPO_ROOT/libraries" ]; then
    LIB_DIR=$REPO_ROOT/libraries
elif [ -d "$REPO_ROOT/../libraries" ]; then
    LIB_DIR=$(CDPATH= cd -- "$REPO_ROOT/../libraries" && pwd)
else
    echo "error: cannot locate libraries/ (set MARGARINE_LIBRARY_DIR)" >&2
    exit 1
fi

CORE_C=$LIB_DIR/core/native/lib.c
STD_C=$LIB_DIR/std/native/lib.c
STD_INCLUDE=$LIB_DIR/std/native

for src in "$CORE_C" "$STD_C"; do
    [ -f "$src" ] || { echo "error: missing $src" >&2; exit 1; }
done

find_tool() {
    for candidate in "$@"; do
        command -v "$candidate" >/dev/null 2>&1 && { printf '%s\n' "$candidate"; return 0; }
    done
    return 1
}

CLANG=$(find_tool clang clang-18 clang-17 clang-16) \
    || { echo "error: clang not found" >&2; exit 1; }

case "$TARGET" in
    wasm32-unknown-unknown)
        ARCHIVER=$(find_tool llvm-ar llvm-ar-18 llvm-ar-17 llvm-ar-16) \
            || { echo "error: llvm-ar not found" >&2; exit 1; }
        ;;
    *)
        ARCHIVER=$(find_tool ar) \
            || { echo "error: ar not found" >&2; exit 1; }
        ;;
esac

mkdir -p "$OUT"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

"$CLANG" --target="$TARGET" -c "$CORE_C" -o "$TMP/core.o"
"$CLANG" --target="$TARGET" -I "$STD_INCLUDE" -c "$STD_C" -o "$TMP/std.o"

"$ARCHIVER" rcs "$OUT/libcore.a" "$TMP/core.o"
"$ARCHIVER" rcs "$OUT/libstd.a" "$TMP/std.o"

printf '%s\n' "built $TARGET: libcore.a libstd.a"
