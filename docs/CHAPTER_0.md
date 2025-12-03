# Chapter 0
## Installation

### Requirements
- **Rust Nightly**: This project requires the Rust nightly toolchain.  
    You can install it by `rustup toolchain install nightly`

```
git clone https://github.com/todaymare/margarine
cd margarine
cargo build --release
```
Awesome, optionally you can add it to your PATH:
```
sudo cp target/release/margarine /usr/local/bin
```

## Quick Start: Hello, margarine!
We'll start by creating our first margarine file, [`hello.mar`](./examples/hello.mar).  
Inside it we can put
```rs
extern "https://github.com/todaymare/margarine-std" as std;
use std::*;

fn main() {
    print("hello, margarine!");
}
```

Then, we can run it with
```sh
margarine run hello.mar
```

Okay so what is happening in that file, you might rightfully ask.

For anyone familiar with Rust this should look pretty normal, except for the elephant in the room.

```rs
extern "https://github.com/todaymare/margarine-std" as std;
```
This is the syntax we use to import any git repository into our program. Depending on the host program these might get ignored or limited but the CLI tool will allow any valid git repo.  

You specify the URL of the git repo and then `as <alias>` in order to name it. 

For now, we just imported the standard library that I made. Maybe later on I should make the standard library being imported the default. Dunno.  

The rest is pretty similar to Rust
```rs
use std::*;
```
Which just imports everything that the `std` library provides. 

```rs
fn main() {
    print("hello, margarine!");
}
```
Defines a function named `main`. The CLI runtime assumes the `main` function is the entry-point.  

You might notice that your program still compiles without it but when running it you'll get an `invalid entry point 'main'` error since the default runtime tries to call that.

And voilla! You have your first margarine program! Don't worry, there's a LOT more to margarine than just Rust without the borrow checker, but before I tell you about that we should probably move to a more structured format.  
