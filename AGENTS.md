# Margarine compiler guidance

## Non-negotiable rules

- Margarine continues compilation after diagnostics. Compiler errors alone must
  never cause an early exit, abort, or code-generation guard.
- Do not run `cargo fmt`; preserve the repository's existing formatting style.
  For multi-line right-hand-side expressions, put the binding or operator on
  its own line and begin the expression on the next line.
- Read `CONTRIBUTING.md` before committing. Commit messages use
  `<type>(<scope>): <summary>` with a lowercase, imperative or noun summary.

## Required verification

After every change to a project or repository file, run the complete,
unfiltered core language suite from the repository root:

```sh
cargo run -p margarine -- test tests/core.mar
```

Report every failure and its output. Compare against a clean checkout before
attributing an existing failure to the change. Prefer `.mar` tests under
`tests/` for observable language and compiler behavior; use Rust tests for
parser/AST contracts, low-level contracts, and diagnostics or compile failures
that cannot run through the language test runner.
- Release, version, managed-layout, and updater behavior tests live under
  `margarine/tests/`; test them through the public API or CLI instead of adding
  source-embedded tests to those production modules. The updater integration
  suite owns its fake release-server and managed-installation fixtures.

## Compiler and CLI architecture

- `margarine/src/cli/mod.rs` owns the clap command surface, dispatch, typed
  `CliError`, and exit-code rendering. `compile.rs`, `test.rs`,
  `artifacts.rs`, `distribution.rs`, `installation.rs`, `update.rs`, and
  `toolchain.rs` own their named responsibilities.
  `margarine/src/main.rs` is the process boundary and the only
  `std::process::exit` callsite.
- Command implementations return `CliResult`. Exit codes are 0 for success,
  1 for compilation or test failures, 2 for clap misuse, 3 for link/toolchain
  failures, and 4 when `run` cannot represent a child exit status.
- `check` tokenizes, parses, and performs semantic analysis only. `build`,
  `run`, and `test` still codegen after diagnostics. The public pipeline is
  `Compiler` + `CompilationResult`: register the entry with `FileData::open`,
  call `run`, then `check`, then `codegen` as appropriate.
- `build` and `run` share `compile_and_link`; `run` executes the path returned
  by that pipeline and reports the source path.

## Targets, linking, and toolchains

Supported compilation targets are:

- `arm64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `wasm32-unknown-unknown`

Native Linux executables link with `clang` and `libstdc++`; native test
libraries use `.so`. The compiler links system LLVM 18.1 through `llvm-sys`
and vendors libgit2/OpenSSL while using Rustls for HTTP. Native non-Wasm
external ABIs use indirect returns for structs larger than 16 bytes.
macOS release compiler builds statically embed zstd, resolve unwind through
Apple's SDK and `libSystem`, and reject every non-system dynamic dependency.
Apple system libraries and frameworks remain dynamic.

`core` and `std` runtime archives are built from the C sources under
`margarine/runtime/core/` and `margarine/runtime/std/` by
`scripts/build-toolchains.sh`. The Wasm `libcore.a` also bundles the vendored
dlmalloc implementation through its linear-memory growth adapter; it does not
depend on WASI or a host-provided libc allocator. Release toolchains place the
resulting archives under `toolchains/<target>/libs/`;
`MARGARINE_TOOLCHAIN_DIR` overrides the root. Every native and Wasm link
searches that directory and passes each regular file there as an explicit
input. Source packages are fetched from the published CDN `share/` tree by
default; `MARGARINE_DEFAULT_URL` overrides the base for ordinary `pkg:`
imports, and `MARGARINE_PRELUDE` overrides preludes. The `git2` dependency
must keep its `https` feature enabled because those package URLs are cloned
over HTTPS; `reqwest` remains the Rustls-only HTTP client.

Ordinary compiler commands work from unmanaged or source-built binaries.
`update` and `toolchain add` require a managed installation. Initial release
installation is owned exclusively by `scripts/install.sh`; there is no
`margarine install` command. The script downloads and checksum-validates the
complete compiler and host-toolchain archives, rejects unsafe archive entries,
stages both under the versioned layout, verifies the compiler before and after
activation, publishes atomically, and rolls back on failure. The updater uses
the same versioned layout and complete compiler archives. Versioned
installation directories and release tags use full SemVer, including
prerelease and build metadata; update comparisons use SemVer precedence.

Native generated entry points use the C `main(int, char **)` ABI and forward
`argc`/`argv` to the standard-library runtime before startup functions run;
Wasm entry points remain zero-argument exports.
`update` locks the installation before network access.
`MARGARINE_RELEASES_API` may point at a compatible test or mirror API.

The `.github/workflows/release.yml` workflow creates a draft GitHub release,
then validates its downloaded installation assets on all three native hosts.
It provides the native test linker (Linux installs `clang`), runs
`tests/core.mar` through the installed binary, and installs `lld-18` plus the
LLVM 18 `wasm-ld` path before building a Wasm example. Native test libraries
use only the runtime archives and platform libraries; they do not require zstd.
The test runner does not support executing tests for the
`wasm32-unknown-unknown` target. Draft assets are downloaded with authenticated
`gh`, checksum-checked, and staged locally for the Wasm test because the
installed compiler's ordinary release lookup cannot see draft releases. Its
macOS compiler build installs LLVM 18 and zstd, statically links zstd, resolves
unwind through Apple's SDK, and rejects non-system dependencies before
packaging the draft compiler as a single executable.

## Caches, packages, and progress

- `check`, `build`, `run`, `test`, and `clean` accept `--cache <dir>` and hold
  the project-root `artifacts.lock` for the whole run. The lock stays outside
  the cache. Cache reset is `--update`; cache deletion is `clean`.
- Repositories live at `<cache>/repos/<hash>`. Each compilation gets a clean
  `<cache>/build/<hash>/` containing `package/`, `out/`, and nested compiler
  cache data. Invalid repository cache entries must be removed and recloned.
- Imported packages produce link inputs through direct `build.mar` scripts.
  Build scripts compile in an isolated in-process workspace, run for the host
  target, receive the captured parent environment and margarine metadata, and
  emit `margarine:link=...` paths relative to `package/`. Build-script errors
  and materialization failures are typed compiler errors, not panics.
- The vendored `sti` dependency lives under `margarine/vendor/sti`; Cargo
  path dependencies resolve from that location.
- All CLI status output is centralized in `margarine/src/progress.rs`.
  Use `StatusLine`, `byte_progress`/`ProgressReader`, and `item_progress`
  instead of command-local progress implementations. Hide progress on
  non-TTY streams and always finish or clear the lifecycle object.

## Semantic and code-generation invariants

- `type` declarations are aliases only; recursive and invalid aliases report
  errors at their definitions. Qualified type conversion preserves requester
  scope, so namespace qualification cannot bypass private members.
- Error types are fresh `SymbolKind::Error(ErrorId)` symbols, not sentinels.
  Type resolution materializes errors directly, error identity is recursive
  through generic arguments, and every error-producing helper records its
  originating `NodeId` without duplicate nodes or backfilled error nodes.
  Two-pass collection may see pending symbols and must use tolerant accessors.
  Resolution continues through error symbols so cascading diagnostics remain
  visible; codegen propagates the recorded diagnostic instead of panicking.
- Error-typed values must not materialize invalid external ABIs. Struct and
  enum codegen may zero-fill only the selected error-typed field or payload;
  omitted non-error struct fields remain semantic errors. Match bindings must
  be popped on every branch path, including codegen errors.
- Trait declarations and implementations retain receiver, implementation, and
  method generics. Validate generic bounds at declarations and preserve
  inout markers in function types, closures, and trait signatures. Missing
  method suggestions come only from traits implemented by the receiver type.
- Trait implementations report a semantic diagnostic when a required method name
  resolves to a non-function declaration; they must not panic during checking.
- `let` is immutable and `var` is mutable. Parameters, loop bindings, match
  bindings, and closure parameters are mutable; closure captures are immutable.
  By-value parameters are owned copies; inout parameters and captures remain
  borrowed according to the existing lowering rules.
- Collection and `str` copies rebuild the LLVM value from `margarineRcClone`'s
  return pointer and restore collection tags. RC clone/drop helpers remain
  eligible for LLVM optimization; ownership correctness must come from emitted
  ownership paths, not from suppressing valid clone/drop elimination.
- The `for-in` statement owns its iterator expression only when the
  expression is a temporary; when it names a place (variable, field, index
  chain), the value still belongs to its owner and the loop must not emit a
  second `emit_drop` for it, or the owner's local-drop releases the same
  value twice. The mutation copy for inout receivers is separate and always
  dropped by the loop itself.
- `ListIter<T>` is an inline aggregate with independent cursor state. Collection
  metadata packs length and rope depth; deep rope concatenation and slicing
  flatten beyond 16 levels. Native string consumers receive a byte pointer and
  explicit length. `$float_sqrt` lowers directly to `llvm.sqrt`.
- Numeric tuple access uses separate dot and integer tokens, including chains.
  The entry file is wrapped in a synthetic root module named after its stem,
  and the generated entry function must claim the LLVM symbol `main` first.
- Function types inside annotations written in generic scopes must carry the
  enclosing generics as their own generic parameters. Without this, a generic
  name inside `fn(...)` resolves to the bare generic container symbol, which
  nothing later substitutes and codegen panics on.
- Function types are structurally equal and structurally hashed. Function
  implementation caching uses a separate nominal instance hash.
- Source-level external file/resource declarations are unsupported. Ordinary
  imported packages use `build.mar`; `extern { ... }` remains the function-ABI
  declaration form.
- `margarine/runtime/margarine.h` is the single C runtime ABI declaration
  surface shared by core and std. `MargarineCollection` and `MargarineString`
  use `int64_t` length fields to match compiler IR on every target; keep
  allocation and refcount internals in `size_t`. Supported Wasm aggregate
  extern returns follow Clang's flattened C ABI: exactly one scalar value
  returns directly; aggregates with multiple values use indirect `sret`.

## Documentation and coverage

- `README.md` is the public overview and quick start. Keep its supported
  targets, CLI examples, source-build prerequisites, contribution link, MIT
  license notice, NASA banner attribution, and linked paths synchronized with
  the repository and release workflow. Keep snippets runnable.
- `scripts/coverage/coverage.sh` is the supported coverage entrypoint. Its
  workflow is documented in `scripts/coverage/SKILL.md`; it uses an isolated
  `target/coverage` build and writes reports and history under
  `artifacts/coverage/`. Do not change compiler behavior or exclusions merely
  to increase coverage.
- The AFL harness lives in `scripts/fuzz/`; build it with `cargo afl build -p
  fuzz` because plain Cargo does not link the AFL runtime.
- Runnable examples belong in `examples/`; keep the repository root free of
  standalone example programs.

The repository is licensed under MIT; `LICENSE` is authoritative.
