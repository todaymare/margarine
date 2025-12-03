
# Chapter 1
## Setting Up a New Project
To set up a new projects, go to any directory you'd like and just create a file. Doesn't even need to end with `.mar`, though that is preferred. I gotta brand this somehow.  

## Making a Guessing Game
```rs
extern "https://github.com/todaymare/margarine-std" as std;
use std::io;

fn main() {
    println("Guess the number:");
    println("Please input your guess.");

    var guess = io::read_line()!;

    print("You guessed: ");
    println(guess);
}
```

You should already be familiar with majority of this but there are a few pieces.

```rs
use std::io;
```
This is used to bring just one symbol from the module

```rs
var guess = io::read_line()!;
```
I really like this syntax. What's happening here is that `read_line` returns a
`Result<str, str>` and the `!` operator "unwraps" it.
