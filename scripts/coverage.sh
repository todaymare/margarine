#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COVERAGE_DIR=${MARGARINE_COVERAGE_DIR:-"$ROOT/artifacts/coverage"}
TARGET_DIR=${CARGO_TARGET_DIR:-"$ROOT/target/coverage"}
TEST_PATH=${MARGARINE_COVERAGE_TESTS:-tests/core.mar}
FILTER=

usage() {
    cat <<EOF
Usage: $0 [--filter NAME] [--tests PATH]

Builds an instrumented Margarine compiler in an isolated target directory,
runs the Margarine test suite, merges LLVM profiles, and writes reports to:
  artifacts/coverage/coverage.json
  artifacts/coverage/coverage.lcov
  artifacts/coverage/report.txt
  artifacts/coverage/uncovered.json
  artifacts/coverage/dashboard.json
  artifacts/coverage/dashboard/index.html
Options:
  --filter NAME   Run only tests whose name contains NAME.
  --tests PATH    Compile PATH instead of tests/core.mar.
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --filter)
            [ "$#" -ge 2 ] || { echo "missing value for --filter" >&2; exit 2; }
            FILTER=$2
            shift 2
            ;;
        --tests)
            [ "$#" -ge 2 ] || { echo "missing value for --tests" >&2; exit 2; }
            TEST_PATH=$2
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

find_tool() {
    tool=$1
    if [ -n "${LLVM_TOOLS_BIN:-}" ] && [ -x "$LLVM_TOOLS_BIN/$tool" ]; then
        printf '%s\n' "$LLVM_TOOLS_BIN/$tool"
        return 0
    fi
    if command -v xcrun >/dev/null 2>&1; then
        path=$(xcrun --find "$tool" 2>/dev/null || true)
        if [ -n "$path" ] && [ -x "$path" ]; then
            printf '%s\n' "$path"
            return 0
        fi
    fi
    command -v "$tool" 2>/dev/null || {
        echo "required LLVM tool not found: $tool (install llvm-tools-preview or LLVM)" >&2
        exit 1
    }
}

LLVM_PROFDATA=$(find_tool llvm-profdata)
LLVM_COV=$(find_tool llvm-cov)

mkdir -p "$COVERAGE_DIR"
for path in \
    "$COVERAGE_DIR"/dashboard \
    "$COVERAGE_DIR"/margarine.profdata \
    "$COVERAGE_DIR"/coverage.json \
    "$COVERAGE_DIR"/coverage.lcov \
    "$COVERAGE_DIR"/report.txt \
    "$COVERAGE_DIR"/uncovered.json \
    "$COVERAGE_DIR"/dashboard.json \
    "$COVERAGE_DIR"/*.profraw
do
    if [ -e "$path" ]; then
        rm -rf "$path"
    fi
done
mkdir -p "$COVERAGE_DIR/history"

RUSTFLAGS_VALUE=${RUSTFLAGS:-}
if [ -n "$RUSTFLAGS_VALUE" ]; then
    RUSTFLAGS_VALUE="$RUSTFLAGS_VALUE -C instrument-coverage -Z coverage-options=branch"
else
    RUSTFLAGS_VALUE="-C instrument-coverage -Z coverage-options=branch"
fi

printf '%s\n' "building instrumented compiler..."
(
    cd "$ROOT"
    CARGO_TARGET_DIR="$TARGET_DIR" \
    CARGO_INCREMENTAL=0 \
    RUSTFLAGS="$RUSTFLAGS_VALUE" \
    cargo build --package margarine --bin margarine
)

PROFILE_PATTERN="$COVERAGE_DIR/margarine-%p-%m.profraw"
BINARY="$TARGET_DIR/debug/margarine"

printf '%s\n' "running Margarine tests..."
(
    cd "$ROOT"
    if [ -n "$FILTER" ]; then
        LLVM_PROFILE_FILE="$PROFILE_PATTERN" "$BINARY" test "$TEST_PATH" "$FILTER"
    else
        LLVM_PROFILE_FILE="$PROFILE_PATTERN" "$BINARY" test "$TEST_PATH"
    fi
)

set -- "$COVERAGE_DIR"/*.profraw
if [ ! -e "$1" ]; then
    echo "no LLVM profile data was produced" >&2
    exit 1
fi

printf '%s\n' "merging LLVM profiles..."
"$LLVM_PROFDATA" merge -sparse "$COVERAGE_DIR"/*.profraw -o "$COVERAGE_DIR/margarine.profdata"

printf '%s\n' "exporting coverage..."
"$LLVM_COV" export "$BINARY" \
    -instr-profile="$COVERAGE_DIR/margarine.profdata" \
    -format=text \
    > "$COVERAGE_DIR/coverage.json"
"$LLVM_COV" export "$BINARY" \
    -instr-profile="$COVERAGE_DIR/margarine.profdata" \
    -format=lcov \
    > "$COVERAGE_DIR/coverage.lcov"
python3 "$ROOT/scripts/coverage_analyze.py" \
    "$COVERAGE_DIR/coverage.json" \
    --source-root "$ROOT" \
    --lcov "$COVERAGE_DIR/coverage.lcov" \
    --output-json "$COVERAGE_DIR/uncovered.json" \
    --output-text "$COVERAGE_DIR/report.txt"

STAMP=$(date -u +%Y-%m-%dT%H-%M-%SZ)
cp "$COVERAGE_DIR/uncovered.json" "$COVERAGE_DIR/history/$STAMP.json"
python3 "$ROOT/scripts/coverage_dashboard.py" \
    --coverage-dir "$COVERAGE_DIR" \
    --output-json "$COVERAGE_DIR/dashboard.json" \
    --output-html "$COVERAGE_DIR/dashboard/index.html"

cat "$COVERAGE_DIR/report.txt"
