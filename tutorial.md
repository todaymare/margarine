# Margarine Language Tutorial

Welcome to the Margarine language tutorial! This guide will take you from the basics to advanced features, with runnable examples. To run any example, save it to a file (e.g., `hello.mar`) and run:

```
cargo r run hello.mar
```

---

## 1. Hello World
```mar
fn main() {
    print("Hello, world!");
}
```

---

## 2. Variables and Types
```mar
var x = 42;
var y: float = 3.14;
var name = "Margarine";
```

---

## 3. Functions
```mar
fn add(a: int, b: int) -> int {
    return a + b;
}

fn main() {
    print(add(2, 3));
}
```

---

## 4. Control Flow
### If/Else
```mar
fn main() {
    var x = 10;
    if x > 5 {
        print("x is large");
    } else {
        print("x is small");
    }
}
```

### Match
```mar
fn main() {
    var x = 2;
    match x {
        1 => print("one"),
        2 => print("two"),
        _ => print("other"),
    }
}
```

---

## 5. Loops
```mar
fn main() {
    for i in 0..5 {
        print(i);
    }
    var x = 0;
    while x < 3 {
        print(x);
        x += 1;
    }
    loop {
        break;
    }
}
```

---

## 6. Structs and Enums
### Structs
```mar
struct Point { x: int, y: int }

fn main() {
    var p = Point { x: 1, y: 2 };
    print(p.x);
}
```

### Enums
```mar
enum Option<T> { Some: T, None }

fn main() {
    var x = Option::Some(42);
    match x {
        Option::Some(val) => print(val),
        Option::None => print("none"),
    }
}
```

---

## 7. Traits and Impls
```mar
trait Display { fn show(self) -> str; }

struct User { name: str }

impl Display for User {
    fn show(self) -> str {
        return self.name;
    }
}

fn main() {
    var u = User { name: "Alice" };
    print(u.show());
}
```

---

## 8. Generics
```mar
fn identity<T>(x: T) -> T { return x; }

fn main() {
    print(identity(123));
    print(identity("abc"));
}
```

---

## 9. Modules and Imports
```mar
mod math;

fn main() {
    print(math::add(1, 2));
}
```

---

## 10. Extern and FFI
```mar
extern "https://example.com/repo" as ext;
use ext::lib;

fn main() {
    print(lib::external_function());
}
```

---

## 11. Pattern Matching and Destructuring
```mar
fn main() {
    var (a, b) = (1, 2);
    print(a);
    print(b);
}
```

---

## 12. Attributes
```mar
@startup fn main() {
    print("Program started!");
}
```

---

## 13. Error Handling
Compile-time errors are reported with source locations. For runtime errors, use `Option`, `Result`, and pattern matching.

---

## 14. Advanced: Closures, Lists, and More
```mar
fn main() {
    var add = |a, b| a + b;
    print(add(2, 3));
    var xs = [1, 2, 3];
    print(xs[0]);
}
```

---

## 15. Full Example
```mar
struct Counter { value: int }

impl Counter {
    fn inc(self) -> Counter {
        return Counter { value: self.value + 1 };
    }
}

fn main() {
    var c = Counter { value: 0 };
    c = c.inc();
    print(c.value);
}
```

---

For more, see the [reference](docs.md) and [examples](../examples/).
