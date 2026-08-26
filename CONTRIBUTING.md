# Contributing

## Commit messages

Format:

```
<type>(<scope>): <summary>
```

- `<type>` is one of:
  - `feat` — new language or compiler feature
  - `fix` — bug fix
  - `refactor` — behavior-preserving restructuring
  - `perf` — performance work
  - `chore` — maintenance (warning cleanup, tooling, vendor updates)
  - `migration` — staged rewrite that leaves the tree in a transitional state until follow-up commits land
- `<scope>` is optional. Use it when the change is contained to one subsystem (`codegen`, `sema`, `parser`, `llvm`, ...); omit it when the change spans several.
- `<summary>` is lowercase, a short imperative or noun phrase, no trailing period, under ~72 characters.

Examples from history:

```
fix: alias errors
feat: $float_sqrt intrinsic
perf: collapse slice of a slice into a slice on the base collection
chore: eliminate warnings in vendor sti
refactor: merge callfunction and calltraitfunction paths
fix(codegen): canonicalize types through ty::resolve
```

A body is optional; add one only when the summary cannot carry the context (for example a non-obvious root cause or a multi-step change). Wrap it at 72 characters.
