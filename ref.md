# Margarine Language Reference

## Introduction
Margarine is a statically-typed, compiled programming language designed for expressiveness, safety, and performance. This document provides a comprehensive reference to every feature of the language, including syntax, semantics, types, modules, generics, and more.

---

## Table of Contents
- [Lexical Structure](#lexical-structure)
- [Literals](#literals)
- [Types](#types)
- [Variables and Patterns](#variables-and-patterns)
- [Expressions](#expressions)
- [Statements](#statements)
- [Functions](#functions)
- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Impls](#impls)
- [Modules](#modules)
- [Generics](#generics)
- [Pattern Matching](#pattern-matching)
- [Attributes](#attributes)
- [Error Handling](#error-handling)
- [Standard Library](#standard-library)
- [Foreign Functions and Extern](#foreign-functions-and-extern)
- [Build System and Imports](#build-system-and-imports)

---

## Lexical Structure
### Keywords
`fn`, `struct`, `impl`, `extern`, `use`, `type`, `mod`, `enum`, `match`, `if`, `else`, `var`, `loop`, `while`, `return`, `break`, `continue`, `as`, `for`, `in`, `trait`, `static`

### Operators and Punctuation
`+`, `-`, `*`, `/`, `%`, `=`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, `&`, `|`, `^`, `<<`, `>>`, `+=`, `-=`, `*=`, `/=`, `%=`, `->`, `=>`, `:`, `::`, `,`, `;`, `.`, `..`, `@`, `?`, `~`, `$`, `(`, `)`, `{`, `}`, `[`, `]`

### Comments
- Single-line: `// comment`

---

## Literals
- Integer: `42`, `0b1010`, `0o77`, `0xFF`
- Float: `3.14`, `2.0e10`
- String: `"hello"`, supports escapes (`\n`, `\t`, `\u{1F600}`)
- Boolean: `true`, `false`
- Unit: `()`

---

## Types
- Primitive: `i64`, `f64`, `bool`, `str`, `unit`, `never`
- Custom: `struct`, `enum`, user-defined
- Tuples: `(T1, T2, ...)`
- Lists: `[T]`
- Function types: `fn(T1, T2) -> T3`
- Generics: `Type<T>`, `fn<T>(...)`
- Type holes: `_`

---

## Variables and Patterns
- Declaration: `var x = expr;`
- Type annotation: `var x: Type = expr;`
- Destructuring: `var (a, b) = expr;`
- Patterns: variable, tuple

---

## Expressions
- Literals, identifiers, parenthesized
- Binary/unary operations
- Function calls: `f(x, y)`
- Field access: `obj.field`
- Indexing: `arr[i]`
- Blocks: `{ ... }`
- If/else: `if cond { ... } else { ... }`
- Match: `match x { ... }`
- Loops: `loop { ... }`, `for x in y { ... }`, `while cond { ... }`
- Return, break, continue
- Struct/enum construction: `Type { ... }`, `Enum::Variant`
- Tuple/list construction: `(a, b)`, `[a, b, c]`
- Casts: `x as Type`
- Closures: `|args| expr`

---

## Statements
- Variable declaration: `var ...`
- Assignment: `x = y`, `x += y`, etc.
- Expression statement: `expr;`
- Block statement: `{ ... }`
- Control flow: `if`, `match`, `loop`, `for`, `while`, `break`, `continue`, `return`

---

## Functions
- Declaration: `fn name(args) -> RetType { ... }`
- Arguments: `name: Type`
- Return type: `-> Type`
- Generics: `fn<T>(...)`
- Methods: `impl Type { fn ... }`
- Closures: `|args| expr`

---

## Structs
- Declaration: `struct Name { field: Type, ... }`
- Generics: `struct Name<T> { ... }`
- Instantiation: `Name { field: value }`
- Field access: `obj.field`

---

## Enums
- Declaration: `enum Name { Variant, Variant2: Type, ... }`
- Generics: `enum Name<T> { ... }`
- Instantiation: `Name::Variant`, `Name::Variant2(value)`
- Pattern matching: `match x { Name::Variant => ... }`

---

## Traits
- Declaration: `trait Name { fn ... }`
- Methods: signatures only
- Implementation: `impl Trait for Type { ... }`

---

## Impls
- Declaration: `impl Type { ... }`
- For trait: `impl Trait for Type { ... }`
- Methods, associated functions

---

## Modules
- Declaration: `mod name { ... }`
- Import: `mod name;`
- Use: `use path::{item, ...}`
- Nested modules

---

## Generics
- Type parameters: `<T, U: Trait>`
- Bounds: `T: Trait`
- Generic functions, structs, enums, traits, impls

---

## Pattern Matching
- `match` expressions
- Patterns: variable, tuple, enum variant
- Guards and bindings

---

## Attributes
- Syntax: `@attribute decl`
- Example: `@startup fn main() { ... }`

---

## Error Handling
- Compile-time errors: syntax, type, semantic
- Error reporting with source ranges

---

## Standard Library
- Core types: `i64`, `f64`, `bool`, `str`, `unit`, `Option`, `Result`, `List`, etc.
- Common functions: `print`, `len`, etc.

---

## Foreign Functions and Extern
- `extern { ... }` for FFI
- Importing from repositories: `extern "url" as alias`
- Importing files: `mod name;`

---

## Build System and Imports
- Imports: `use`, `mod`, `extern`
- Artifacts and build.lock
- Running: `cargo r run <file>`

---

## Full Syntax Reference
(See above sections for details. For edge cases and advanced usage, see the tutorial.)

---

## See Also
- [Tutorial](tutorial.md)
- [Examples](../examples/)
