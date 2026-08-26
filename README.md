<p align="center">
  <img
    src="./assets/banner.jpg"
    width="100%"
    alt="Margarine banner"
  >
</p>

# margarine

A programming language with simple syntax, simple semantics,  
and LLVM-based native and WebAssembly compilation.

[![Version](https://img.shields.io/github/v/tag/todaymare/margarine?label=version)](https://github.com/todaymare/margarine/tags)
[![License](https://img.shields.io/github/license/todaymare/margarine)](./LICENSE)
[![Stars](https://img.shields.io/github/stars/todaymare/margarine?style=flat&label=stars)](https://github.com/todaymare/margarine/stargazers)
[![Release](https://img.shields.io/github/actions/workflow/status/todaymare/margarine/release.yml?label=release)](https://github.com/todaymare/margarine/actions/workflows/release.yml)


## Overview
`margarine` is a statically typed, Rust-inspired programming language with
traits, generics, enums, pattern matching, closures, iterators, and explicit
mutability—without a borrow-checker-driven programming model.

Its LLVM-based compiler continues through diagnostics and can produce a usable
binary even when source errors remain. If execution reaches an error path,
Margarine traps there with the corresponding diagnostic.


## Install

On supported systems:

```sh
curl -fsSL https://cdn.daymare.net/margarine/install.sh | sh
```

> [!NOTE]
> Windows is not currently supported. The compiler presently relies on Unix-specific functionality in parts of the compiler and toolchain.

## Supported targets

| Target | Output |
| --- | --- |
| `arm64-apple-darwin` | Native executable |
| `x86_64-unknown-linux-gnu` | Native executable |
| `aarch64-unknown-linux-gnu` | Native executable |
| `wasm32-unknown-unknown` | WebAssembly module |

## Project status

`margarine` is still early-stage. Patch releases aim to remain backwards-compatible within a minor version, while minor releases may introduce breaking changes to the language, standard library, or tooling.  
This may change after version 1.0.0.

## Usage

A simple Margarine program may look like this:

```rs
fn main() {
    println("Hello!");
}
```

*For more examples, see the [examples folder](examples).*

## CLI quick reference

```sh
margarine run examples/hello.mar
margarine build examples/hello.mar
margarine check examples/hello.mar
margarine test tests/core.mar
margarine update
margarine toolchain add wasm32-unknown-unknown
```


## Building from source

Building from source requires Rust nightly, LLVM 18, and `clang`.

On macOS:

```sh
brew install llvm@18
export LLVM_SYS_181_PREFIX="$(brew --prefix llvm@18)"
```

On Debian or Ubuntu:

```sh
sudo apt-get install llvm-18 llvm-18-dev libpolly-18-dev clang-18
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18
```

Then build and run the compiler from the repository root:

```sh
git clone https://github.com/todaymare/margarine
cd margarine
cargo +nightly build -p margarine
cargo +nightly run -p margarine -- run examples/hello.mar
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development and commit-message
conventions.

## License

This project is available under the [MIT License](LICENSE).

## Image credit

Banner image: [Shuttle Over Earth](https://apod.nasa.gov/apod/ap220417.html),
NASA / Expedition 22 Crew, via NASA's Astronomy Picture of the Day.
