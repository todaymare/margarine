# Contributing
Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are greatly appreciated.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement". Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (git checkout -b feature/AmazingFeature)
3. Commit your Changes (git commit -m 'feat: some amazing feature')
4. Push to the Branch (git push origin feature/AmazingFeature)
5. Open a Pull Request

Keep changes focused on the problem you’re solving, test them thoroughly, and prefer small, coherent commits over large mixed changes.

## Testing

Before opening a pull request, make sure both test suites pass:

```sh
cargo test
margarine test tests/core.mar
```

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

## AI-assisted contributions

AI-assisted contributions are allowed, but contributors are responsible for everything they submit. Do not open pull requests containing code you do not understand or cannot maintain.

AI-generated changes must be reviewed, tested, and held to the same standards as handwritten changes. Contributions that appear to be bulk-generated without understanding, validation, or appropriate tests may be rejected.

If AI was used substantially in preparing a change, disclosure in the pull request is appreciated but not required.
