# Margarine — A Guided Tour

Margarine feels like Rust without the fights you used to lose with the borrow
checker. This tour takes you from zero to a working guessing game, one small
step at a time. Every snippet in here compiles and runs against the current
compiler and standard library.

To run any example, save it to a file (say `hello.mar`) and:

```sh
margarine run hello.mar
```

(or `cargo run -p margarine -- run hello.mar` from the compiler repo).

---

## 1. Hello, margarine!

```mar
import "pkg:std" as std;
use std::*;

fn main() {
    println("hello, margarine!");
}
```

Two lines deserve attention before the obvious one.

```mar
import "pkg:std" as std;
```

This pulls the `std` package from the default package system. The string can be
any git URL; the `pkg:` prefix is a shortcut for the default registry, which is
controlled by the `MARGARINE_DEFAULT_URL` environment variable (it defaults to
`pkg.daymare.net/margarine`). You name the import with `as <alias>`.

```mar
use std::*;
```

Brings everything `std` exports into scope. You can also import specific
modules — more on that below.

Finally:

```mar
fn main() { ... }
```

defines the entry point. The CLI runtime calls `main`. Your program will still
compile without it, but running it fails with `invalid entry point 'main'`.

> **Note:** unlike Rust, compilation continues after errors. If something
> upstream failed, you may see a wall of cascading diagnostics followed by a
> program that still links. Read the *first* error first.

---

## 2. Variables: `var` and `let`

```mar
import "pkg:std" as std;
use std::*;

fn main() {
    var x = 42;
    x += 1;              // var bindings are mutable

    let y = x * 2;       // let bindings are immutable
    // y += 1;           // error: cannot assign to immutable binding

    println(x);
    println(y);
}
```

- `var` declares a mutable variable.
- `let` declares an immutable binding.
- Types are usually inferred; annotate when you want to pin them down:
  `var n: int = 0;`.

---

## 3. Functions

```mar
import "pkg:std" as std;
use std::*;

fn add(a: int, b: int): int {
    a + b                 // last expression is returned
}

fn greet(name: str): str {
    "hello, ".concat(name)
}

fn main() {
    println(add(2, 3));
    println(greet("margarine"));
}
```

Function syntax follows Rust, except the return type uses `:` instead of `->`,
and there is no `return` keyword needed for the tail expression.

### In-out parameters

Functions can take parameters by reference-and-writeback with `&`:

```mar
import "pkg:std" as std;
use std::*;

fn increment(&value: int) {
    value = value + 1;
}

fn main() {
    var n = 1;
    increment(&n);
    println(n);           // 2
}
```

Call sites must mark in-out arguments with `&`, and the argument must be an
assignable place (a variable, field, or index) — not a temporary.

---

## 4. Control flow

```mar
import "pkg:std" as std;
use std::*;

fn main() {
    var x = 10;

    if x > 5 {
        println("big");
    } else if x == 5 {
        println("five");
    } else {
        println("small");
    }

    var i = 0;
    while i < 3 {
        println(i);
        i += 1;
    }

    loop {                // loops forever until `break`
        i += 1;
        if i >= 10 { break; }
    }
    println(i);
}
```

Ranges pair naturally with `for`:

```mar
import "pkg:std" as std;
use std::*;

fn main() {
    for i in 0..5 {          // exclusive upper bound
        println(i);
    }

    for i in 0..=5 {         // inclusive upper bound
        println(i);
    }
}
```

`if` is an expression too, so this works:

```mar
import "pkg:std" as std;
use std::*;

fn max(a: int, b: int): int {
    if a > b { a } else { b }
}

fn main() {
    println(max(3, 9));
}
```

---

## 5. Structs

```mar
import "pkg:std" as std;
use std::*;

struct Point {
    x: int,
    y: int,
}

impl Point {
    fn new(x: int, y: int): Self {
        Self { x, y }
    }

    fn manhattan(self): int {
        self.x.abs() + self.y.abs()
    }
}

fn main() {
    var p = Point::new(3, -4);
    println(p.manhattan());      // 7
    println(p.x);                // 3
}
```

Methods live in `impl` blocks. `Self` refers to the enclosing type. Methods
taking `self` consume a copy of the value; use `&self` for in-out methods that
mutate through the receiver.

---

## 6. Enums and match

Enums define tagged unions. Constructors are namespaced under the enum name:

```mar
import "pkg:std" as std;
use std::*;

enum Shape {
    Circle(int),
    Rect((int, int)),
}

fn area(s: Shape): int {
    match s {
        Circle(r) => 3 * r * r,
        Rect(dims) => dims.0 * dims.1,
    }
}

fn main() {
    var shapes = [Shape::Circle(2), Shape::Rect((3, 4))];
    for s in shapes.iter() {
        println(area(s));
    }
}
```

Notes:

- Construct with `Shape::Circle(2)`; pattern-match with the bare variant name.
- Multi-value payloads wrap the tuple explicitly: `Rect((int, int))`.
- `match` patterns bind enum variants. Literal patterns are not supported yet —
  use `if` chains for value comparisons.

### Option and Result

`Option<T>` and `Result<T, E>` are built in, with `some(...)` / `none()` and
`ok(...)` / `err(...)` constructors:

```mar
import "pkg:std" as std;
use std::*;

fn find_user(id: int): Option<str> {
    if id == 1 { some("ada") } else { none() }
}

fn main() {
    match find_user(1) {
        some(name) => println(name),
        none => println("not found"),
    }
}
```

Unwrapping:

- `opt!` unwraps an `Option` and panics on `none`.
- `res!` unwraps a `Result` and panics on `err`.
- `expr?` unwraps or returns early from the enclosing function.
- `f.opt! = v` / `f.opt? = v` assign through unwrapped places.

```mar
import "pkg:std" as std;
use std::*;

struct Config {
    retries: Option<int>,
}

fn load(): Config {
    Config { retries: some(3) }
}

fn retries_or(config: Config, fallback: int): int {
    match config.retries {
        some(n) => n,
        none => fallback,
    }
}

fn main() {
    var config = load();
    config.retries! = 5;             // unwrap-assign
    println(config.retries!);

    var other = Config { retries: none() };
    println(retries_or(other, 1));
}
```

---

## 7. Traits

Traits declare shared behavior; `impl Trait for Type` implements them:

```mar
import "pkg:std" as std;
use std::*;

trait Describe {
    fn describe(self): str
}

trait Counter {
    fn bump(&self)
}

struct Door {
    open_count: int,
}

impl Describe for Door {
    fn describe(self): str {
        "a door"
    }
}

impl Counter for Door {
    fn bump(&self) {
        self.open_count += 1;
    }
}

fn main() {
    var d = Door { open_count: 0 };
    d.bump();
    d.bump();
    println(d.describe());
    println(d.open_count);       // 2
}
```

Trait declarations end without braces around bodies — just signatures. Generic
traits (`trait FromStr { ... }`) and generic impls both work; see
`tests/core/traits.mar` in the compiler repo for deeper examples.

---

## 8. Generics

```mar
import "pkg:std" as std;
use std::*;

fn identity<T>(x: T): T {
    x
}

fn replace<T>(&value: T, replacement: T) {
    value = replacement;
}

fn main() {
    println(identity(123));
    println(identity("abc"));

    var n = 1;
    replace(&n, 42);
    println(n);
}
```

Bounds use `:` at the parameter site (`fn parse<T: FromStr>(self): Option<T>`).

Type aliases give names to existing types:

```mar
type UserId = int;
type Pair<T> = (T, T);
```

---

## 9. Tuples, lists, and iterators

Tuples support `.0`/`.1` accessors, including chains like `t.0.1`:

```mar
import "pkg:std" as std;
use std::*;

fn make_pair(): (int, int) {
    (1, 2)
}

fn main() {
    var t = make_pair();
    println(t.0 + t.1);

    let nested = ((3, 4), 5);
    println(nested.0.1);        // 4
}
```

Lists are `[T]`, written literally as `[1, 2, 3]`:

```mar
import "pkg:std" as std;
use std::*;

fn main() {
    var xs = [10, 20, 30];
    xs.push(40);
    println(xs.len());          // 4
    println(xs[2]);             // 30

    var total = 0;
    for x in xs.iter() {        // iterate with .iter()
        total += x;
    }
    println(total);             // 100
}
```

---

## 10. Strings

Strings are UTF-8 slices with byte-oriented indexing:

```mar
import "pkg:std" as std;
use std::*;

fn main() {
    var s = "hello".concat(" world");
    println(s);                     // hello world
    println(s.len());               // 11 (bytes)

    println(s.slice(0..5));         // hello
    println(s.find("world"));       // some(6)
    println(s.contains("mar"));     // false

    for line in "a\nb\nc".iter() {
        println(line);
    }
}
```

Formatting is manual concatenation for now — string templating is still
rough. Integers and floats convert with `.to_str()`, which comes from the
`ToStr` trait, so import the string namespace when you need it:

```mar
import "pkg:std" as std;
use std::*;
use std::string::*;

fn main() {
    var n = 42;
    println("n = ".concat(n.to_str()));
}
```

Iterators chain with `map` / `enumerate` and finish with `sum` / `count`:

```mar
import "pkg:std" as std;
use std::*;

fn double(n: int): int { n * 2 }

fn main() {
    var nums = [5, 3, 8, 1];

    println(nums.iter().map(double).sum());   // 34
    println(nums.iter().count());             // 4

    for i, n in nums.iter().enumerate() {
        if i == 2 { println(n); }             // 8
    }
}
```

Closures are `|args| body`; parameter types can be annotated. Closures passed
to iterator combinators take plain values, while closures standing in for
`fn(&S)` state callbacks take `&`:

```mar
import "pkg:std" as std;
use std::*;

fn main() {
    var m = 3;
    var nums = [5, 3, 8];
    println(nums.iter().map(|n: int| n * m).sum());   // captures m: 48
}
```

Captured values are immutable inside closure bodies — closures read your
locals, they don't borrow them mutably.

---

## 11. Error handling: reading input

`io::read_line()` returns `Result<str, str>` — unwrap with `!`:

```mar
import "pkg:std" as std;
use std::*;

fn main() {
    println("Please input your guess.");

    var guess = io::read_line()!;
    print("You guessed: ");
    println(guess);
}
```

Note that `read_line` keeps the trailing newline. To parse the input as a
number, slice it off and use `parse::<int>()!`:

```mar
import "pkg:std" as std;
use std::*;

fn main() {
    var guess = io::read_line()!;
    var trimmed = guess.slice(0..guess.len() - 1);
    var n = trimmed.parse::<int>()!;
    println(n * 2);
}
```

`parse` returns `Option<T>` where `T: FromStr` — `int` implements `FromStr`,
and the turbofish `::<int>()` selects the target type.

---

## 12. Random numbers

The `rand` module provides a seeded generator. Import its namespace directly:

```mar
import "pkg:std" as std;
use std::*;
use std::rand::*;

fn main() {
    var rng = Rng::new(Xoshiro256::from_seed((1234, 0, 0, 7)));
    for _ in 0..5 {
        println(rng.random_range(0..10));
    }
}
```

Same seed, same sequence — great for tests. For game-style randomness, seed
from the clock. Note that `random_range` can return negative values (it mods
the raw draw), so normalize:

```mar
import "pkg:std" as std;
use std::*;
use std::rand::*;
use std::duration::*;

fn main() {
    var seed = Duration::now().as_nanos();
    var rng = Rng::new(Xoshiro256::from_seed((seed, 0, 0, 7)));
    var n = rng.random_range(0..10);
    if n < 0 { n += 10; }
    println(n);
}
```

Each module needs its own `use`: `use std::*` pulls top-level names only, so
`Duration` requires `use std::duration::*`.

Timing code is the same trick:

```mar
import "pkg:std" as std;
use std::*;
use std::duration::*;

fn main() {
    var start = Duration::now();
    var total = 0;
    for i in 0..1_000_000 { total += i; }
    println(Duration::now().sub(start).to_str_dynamic());
}
```

---

## 13. Modules and visibility

Declarations are private by default; mark them `pub` to export:

```mar
mod math_utils {
    pub fn square(x: int): int {
        x * x
    }

    fn helper(): int {
        1
    }
}

fn main() {
    println(math_utils::square(7));
}
```

Split files with `mod name;` plus a matching file, and import across packages
with `import "<url>" as alias;` followed by `use alias::path`.

---

## 14. Testing your code

The `@test` attribute registers a function with the test runner:

```mar
import "pkg:std" as std;
use std::*;

fn add(a: int, b: int): int {
    a + b
}

@test
fn add_works() {
    assert(add(2, 3) == 5, "addition broke");
}

@test(should_panic)
@silent
fn rejects_bad_input() {
    var empty = "";
    empty.parse::<int>()!;
}
```

Run the suite with:

```sh
margarine test my_file.mar
```

- `assert(cond, msg)` panics with `msg` when `cond` is false.
- `@test(should_panic)` expects the test to panic.
- `@silent` suppresses expected diagnostic noise.
- Filter with `margarine test file.mar <filter>` or the
  `MARGARINE_TEST_FILTER` environment variable.

---

## 15. Putting it together: the guessing game

Everything above, in one program:

```mar
import "pkg:std" as std;
use std::*;
use std::rand::*;
use std::duration::*;

fn main() {
    var seed = Duration::now().as_nanos();
    var rng = Rng::new(Xoshiro256::from_seed((seed, 0, 0, 7)));
    var secret = rng.random_range(0..10);
    if secret < 0 { secret += 10; }

    loop {
        println("Guess the number:");
        println("Please input your guess.");

        var guess = io::read_line()!;
        var trimmed = guess.slice(0..guess.len() - 1);
        var guess_num = trimmed.parse::<int>()!;

        print("You guessed: ");
        println(guess_num);

        if secret < guess_num {
            println("You guessed too high!");
        } else if secret > guess_num {
            println("You guessed too low!");
        } else {
            println("Correct!");
            break;
        }
    }
}
```

A complete runnable copy lives at [`examples/guessing_game.mar`](../examples/guessing_game.mar).
More examples are in [`examples/`](../examples/) — including
[`brainfuck.mar`](../brainfuck.mar), a brainfuck interpreter written in
margarine.

---

## Where to go next

- Browse the standard library sources (`lib/` in the std package) — they are
  ordinary margarine and double as idiomatic examples.
- `tests/core/` in the compiler repo exercises every language feature shown
  here and quite a few more (closures with in-out params, `$rc`, ropes, FFI).
- Compiler internals and contribution workflow: see
  [CONTRIBUTING.md](../CONTRIBUTING.md).
