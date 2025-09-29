# Complete Language Reference

This document provides a complete reference for the language, derived from the provided lexer and parser source files. It is divided into two sections: **Tokens & Literals** (lexical structure) and **Syntax & Grammar** (program structure).

---

## 1. Tokens & Literals
This section describes the fundamental building blocks of the language as recognized by the lexer.

### **1.1 Identifiers & Keywords**
Identifiers are used for variable names, functions, types, and other custom elements.
* **Syntax:** An identifier must begin with an alphabetic character or an underscore (`_`). Subsequent characters can be alphanumeric or an underscore.
* **Reserved Keywords:** The following words are reserved and cannot be used as identifiers:
    * `as`
    * `break`
    * `continue`
    * `else`
    * `enum`
    * `extern`
    * `fn`
    * `for`
    * `if`
    * `impl`
    * `in`
    * `loop`
    * `match`
    * `mod`
    * `return`
    * `struct`
    * `use`
    * `var`
    * `while`

### **1.2 Literals**
Literals represent fixed values in the source code.
* **Integer Literals:** A sequence of digits that can be parsed as a 64-bit integer (`i64`). Example: `123`, `0`, `987654`.
* **Floating-Point Literals:** A sequence of digits containing a single decimal point. These are parsed as 64-bit floating-point numbers (`f64`). Example: `12.34`, `0.5`, `1.0`.
* **Boolean Literals:** The keywords `true` and `false`.
* **String Literals:** A sequence of characters enclosed in double quotes (`"`). The following escape sequences are supported:
    * `\n`: newline
    * `\r`: carriage return
    * `\t`: tab
    * `\\`: backslash
    * `\0`: null character
    * `\"`: double quote
    * `\u{...}`: Unicode escape sequence, where the content inside the braces is a hexadecimal representation of the desired Unicode character.

### **1.3 Operators & Punctuation**
* **Arithmetic:** `+`, `-`, `*`, `/`, `%`
* **Assignment:** `=`, `+=`, `-=`, `*=`, `/=`, `%=`
* **Comparison:** `==`, `!=`, `<`, `>`, `<=`, `>=`
* **Logical:** `&&`, `||`
* **Bitwise:** `|`, `^`, `&`, `<<`, `>>`
* **Structural & Other:** `(`, `)`, `{`, `}`, `[`, `]`, `,`, `.`, `..`, `::`, `:`, `_`, `!`, `?`, `@`, `=>`, `~`

### **1.4 Comments & Whitespace**
* **Single-line comments:** The lexer ignores all content from a `//` to the end of the line.
* **Whitespace:** Spaces, tabs, and newlines are ignored and serve as separators between tokens.

---

## 2. Syntax & Grammar
This section describes how the tokens are combined to form valid program structures.

### **2.1 Program Structure**
A complete program or module consists of a series of top-level declarations. A block containing statements (e.g., variable declarations or expressions) is invalid at the top level.

### **2.2 Declarations**
Declarations introduce new named entities into the program's scope.
* **Function:** Defines a function.
    ```
    fn name<generics>(arg1: Type, arg2: Type) : ReturnType { ... }
    ```
    * **Generics** are optional: `<T>`.
    * **Return type** is optional. If omitted, it defaults to the `Unit` type (`()`).
    * A function body is a required block (`{...}`).
    * Within an `impl` block, a special `self` argument is recognized.
* **Struct:** Defines a custom data type.
    ```
    struct Name<generics> {
        field1: Type,
        field2: Type,
    }
    ```
    * **Generics** and fields are optional.
* **Enum:** Defines a sum type with multiple variants.
    ```
    enum Name<generics> {
        Variant1,
        Variant2: Type,
    }
    ```
    * **Generics** are optional.
    * Each variant can optionally have an associated type, specified with a colon (`:`).
* **Implementation Block:** Adds methods and functions to a `struct` or `enum`.
    ```
    impl<generics> Type { ... }
    ```
    * The body of an `impl` block must contain only declarations (`fn`, `struct`, etc.).
* **Module:** Creates a new nested scope for declarations.
    ```
    mod name { ... }
    ```
* **External Functions:** Declares functions implemented in external code.
    ```
    extern {
        fn name(arg: Type, ...) : ReturnType,
    }
    ```
* **Use Statement:** Imports names from other modules.
    ```
    use path::name;
    use path::{name1, name2, ...};
    use path::*;
    ```
* **Attributes:** Applies an attribute to a declaration.
    ```
    @attribute declaration
    ```

### **2.3 Statements**
Statements perform actions and do not necessarily return a value.
* **Variable Declaration:** Binds a value to a new variable.
    ```
    var name = value;
    var name: Type = value;
    ```
* **For Loop:** Iterates over a range or iterable expression.
    ```
    for binding in expression { ... }
    ```
* **Assignment:** Assigns a new value to a variable or expression.
    ```
    variable = value;
    variable += value; // And other compound operators
    ```
* **Expression Statements:** Any expression can be used as a statement by simply being followed by the end of the line or a semicolon (implicitly handled by the parser).

### **2.4 Expressions**
Expressions produce a value and can be combined. They are listed below in order of precedence, from lowest to highest.

#### **2.4.1 Precedence and Operators**
* **Logical OR:** `||`
* **Logical AND:** `&&`
* **Comparison:** `==`, `!=`, `<`, `>`, `<=`, `>=`
* **Bitwise OR:** `|`
* **Bitwise XOR:** `^`
* **Bitwise AND:** `&`
* **Bitshifts:** `<<`, `>>`
* **Arithmetic:** `+`, `-`
* **Product:** `*`, `/`, `%`
* **Range:** `..` (exclusive), `..=` (inclusive)
* **Unary:** `-` (negation), `!` (logical NOT)

#### **2.4.2 Basic Expressions**
* **Literals:** Any of the integer, float, string, or boolean literals.
* **Identifiers:** A variable or name.
* **Parentheses:** Used for grouping expressions, e.g., `(2 + 3) * 4`.
* **Unit Value:** `()` represents the `Unit` type, which has no value.
* **Tuples:** A comma-separated list of expressions inside parentheses, e.g., `(1, "hello", true)`.

#### **2.4.3 Control Flow Expressions**
* **If/Else:** Evaluates a condition and executes a block.
    ```
    if condition { ... } else { ... }
    ```
    The `else` block is optional and can be another `if` expression.
* **Match:** Compares a value against a series of patterns.
    ```
    match value {
        Variant1 => expr1,
        Variant2: binding => expr2,
    }
    ```
    * Patterns can be `enum` variants or boolean literals.
    * `bind_to` allows a variable to be bound to the inner value of a variant.
* **Loop:** An infinite loop.
    ```
    loop { ... }
    ```
* **While:** A loop that continues as long as a condition is true.
    ```
    while condition { ... }
    ```
* **Return:** Returns a value from a function. `return expression;`
* **Break:** Exits a loop. `break;`
* **Continue:** Skips to the next iteration of a loop. `continue;`
* **Blocks:** A sequence of statements and expressions inside braces. The last expression in a block is the return value of the block.

#### **2.4.4 Function & Method Calls**
* **Function Call:** `name(arg1, arg2, ...)`
* **Method Call (Accessor):** `object.method(arg1, arg2, ...)`

#### **2.4.5 Structs & Tuples**
* **Struct Creation:** Instantiates a new struct.
    ```
    var my_struct = StructType {
        field1: value1,
        field2: value2,
    }
    ```
* **Tuple Creation:** Creates a tuple. `var my_tuple = (expr1, expr2, ...)`
* **Field Access:** Accesses a named field of a struct or a tuple by numeric index.
    ```
    my_struct.field_name
    my_tuple.1 // Accesses the second element
    ```

#### **2.4.6 Casting & Other Accessors**
* **Casting:** Explicitly converts an expression to another type. `expression as Type`
* **Unwrap:** Attempts to unwrap a value, panicking if it's an error. `expression!`
* **Or-Return:** Returns from the current function if the expression is an error. `expression?`

#### **2.4.7 Namespaces**
* **Static/Namespace Access:** `Namespace::item`
* **Type Namespace:** `[Type]::item`


> note: this was generated by gemini. sorry chat i'm lazy

