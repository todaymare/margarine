# margarine

> Imagine a language that feels like Rust, but without the fights I used to lose with the borrow checker. 

The syntax of margarine is very akin to Rust with iterators, closures, and much more!

I've been working on this for years at this point and I'm excited to tell you about it so let's get you started!

## Installation

### Requirements
- **Rust Nightly**: This project requires the Rust nightly toolchain.  
    You can install it by `rustup toolchain install nightly`

```
git clone https://github.com/todaymare/margarine
cd margarine
cargo build --release
```
After building, you can manage the compiler with its built-in updater instead of
copying the binary by hand:
```sh
margarine install
```
This installs the current binary into `~/.margarine/<version>/bin/margarine`,
activates it through `~/.margarine/bin/margarine`, downloads the host runtime
toolchain, and prints the `PATH` line to add to your shell profile. Later,
`margarine update` checks for new releases and self-updates in place, and
`margarine toolchain add <target>` fetches runtime libraries for other targets.

## Quick Start: Hello, margarine!
We'll start by creating our first margarine file, [`hello.mar`](./examples/hello.mar).  
Inside it we can put
```mar
import "pkg:std" as std;
use std::*;

fn main() {
    println("hello, margarine!");
}
```

Then, we can run it with
```sh
margarine run hello.mar
```

Okay so what is happening in that file, you might rightfully ask.

For anyone familiar with Rust this should look pretty normal, except for the elephant in the room.

```mar
import "pkg:std" as std;
```
This is how we pull a package into our program. The string can be *any* git URL; the `pkg:` prefix is a shortcut for the default package system, which is controlled by the `MARGARINE_DEFAULT_URL` environment variable (it defaults to `pkg.daymare.net/margarine`).

You specify the URL of the package and then `as <alias>` in order to name it. Here we imported the standard library as `std`.

The rest is pretty similar to Rust
```rs
use std::*;
```
Which just imports everything that the `std` library provides. 

```rs
fn main() {
    println("hello, margarine!");
}
```
Defines a function named `main`. The CLI runtime assumes the `main` function is the entry-point.  

You might notice that your program still compiles without it but when running it you'll get an `invalid entry point 'main'` error since the default runtime tries to call that.

And voilla! You have your first margarine program! Don't worry, there's a LOT more to margarine than just Rust without the borrow checker.

## Learn more

The full guided tour — variables, functions, structs, enums, traits, generics,
iterators, error handling, testing, and a complete guessing game — lives in
[`docs/tutorial.md`](./docs/tutorial.md). More runnable programs are in
[`examples/`](./examples/).

## Compiler coverage

Compiler branch coverage can be measured while keeping the inputs as ordinary
Margarine programs:

```sh
./scripts/coverage.sh
```

The script builds an instrumented compiler in `target/coverage`, runs
`tests/core.mar`, merges the generated LLVM profiles, and writes:

- `artifacts/coverage/coverage.json` — raw `llvm-cov` export data
- `artifacts/coverage/coverage.lcov` — portable `BRDA` branch-edge records
- `artifacts/coverage/report.txt` — edge coverage, branch-group summary, source context, and scope totals
- `artifacts/coverage/uncovered.json` — machine-readable branch-edge targets grouped by source site
- `artifacts/coverage/dashboard.json` — dashboard data for agents and scripts
- `artifacts/coverage/dashboard/index.html` — self-contained interactive dashboard
- `artifacts/coverage/history/` — timestamped analyzer results preserved across runs

Open `artifacts/coverage/dashboard/index.html` locally to filter by first-party
or third-party scope, status, file, and source text. Each target has a
continuation-prompt copy action. The dashboard is static and needs no server.

The report distinguishes branch edges from branch groups. A group is one
source-level conditional with its outgoing edges; groups are useful targets,
but the edge total is the actual coverage denominator.

Coverage is split into:

- `first_party` — Rust compiler sources under this repository, excluding `vendor/`
- `third_party` — vendored or external Rust dependency sources reached by the run

The aggregate remains available, but first-party coverage is the primary
compiler metric because dependency versions and implementations can change.

To run only matching language tests:

```sh
./scripts/coverage.sh --filter generics
```

Use `--tests PATH` to compile a different Margarine test source. The ordinary
Cargo target and build remain separate from the instrumented target. LLVM tools
are discovered from `LLVM_TOOLS_BIN`, Xcode (`xcrun`), or `PATH`.

For an LLM continuation workflow, provide the repository-local skill at
`skills/margarine-compiler-coverage/SKILL.md` and say:

```text
Continue Margarine compiler coverage work from the current dashboard.
```
