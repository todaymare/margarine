# Margarine Compiler Coverage Continuation

Use this skill when asked to continue improving Margarine compiler coverage.

## Objective

Increase genuine compiler behavior exercised by ordinary Margarine programs and
focused compiler-command scenarios. Coverage is a search signal, not the goal
by itself. Never change compiler or runtime behavior solely to increase a
percentage.

## Start with the current state

From the `compiler/` directory:

```sh
./scripts/coverage.sh
```

Inspect:

```text
artifacts/coverage/dashboard/index.html
artifacts/coverage/dashboard.json
artifacts/coverage/uncovered.json
artifacts/coverage/report.txt
```

The dashboard separates first-party compiler code from third-party Rust code.
Use `dashboard.json` and `uncovered.json` for machine-readable decisions; do
not scrape the HTML.

## Continue loop

1. Read the current summary and compare it with the latest history entry.
2. Select an actionable target, prioritizing first-party parser, lexer,
   semantic-analysis, diagnostics, and code-generation behavior.
3. Inspect the referenced Rust source and determine what observable behavior
   the missing edge represents.
4. Find an existing `.mar` test that should cover it, or add a focused test
   under the repository's existing test structure.
5. Assert observable behavior: a value, successful compilation, a diagnostic,
   a rejection, or an expected panic.
6. Run the complete core suite:

   ```sh
   cargo run -p margarine -- test tests/core.mar
   ```

7. Re-run `./scripts/coverage.sh` and confirm that the intended edge changed.
8. Record no success unless the test still expresses meaningful language or
   compiler behavior and the complete suite passes.
9. Repeat while targets remain actionable and testable.

## Scope decisions

- First-party compiler coverage is the primary metric.
- Third-party coverage is useful evidence about end-to-end execution, but is
  reported separately because dependency versions and implementations change.
- Runtime and standard-library coverage require separate instrumentation of the
  generated program; do not merge it into compiler coverage without preserving
  separate scope totals.
- Uncovered CLI, filesystem, cache, network, and malformed-input paths may need
  compiler-command or harness scenarios rather than valid `.mar` programs.
- A target that is unreachable through meaningful behavior may be documented
  as non-actionable; do not add a garbage program to touch it.

## Forbidden shortcuts

Do not:

- modify compiler/runtime implementation merely for coverage;
- add coverage exclusions or mark code unreachable;
- delete tests or weaken assertions;
- suppress diagnostics or alter expected behavior;
- add a source file whose only purpose is touching one arbitrary branch;
- claim progress from a partial or filtered run as full-suite verification.

## Completion

Stop only when the requested coverage scope is exercised, or remaining targets
are explicitly classified as non-actionable with evidence. Report the exact
edge/group totals, changed tests, and commands run. If the user says
“continue,” resume from the current dashboard and history rather than asking
for a new target.
