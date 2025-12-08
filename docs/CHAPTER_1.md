
# Chapter 1
## Setting Up a New Project
To set up a new projects, go to any directory you'd like and just create a file. Doesn't even need to end with `.mar`, though that is preferred. I gotta brand this somehow.  

## Making a Guessing Game
Let's make a guessing game shall we?

### Processing a guess
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
`Result<str, str>` and the `!` operator "unwraps" it. Unwrapping a result will CRASH the program if the result is an error, or if its ok then it'll return that value.

We're also creating a variable here. Variables in margarine are always mutable.

Then we have the prints.
```rs
print("You guessed: ");
println(guess);
```
Yeah, string templating sucks right now. It'll get better. Until then, enjoy the manual concatenation experience. 

### The actual game part?
Okay yeah, you'd need to squint really hard to call what we made a game so let's pretty it up a little, shall we?

We can first start by wrapping it all in a loop so the player can play again without restarting the whole program.

```rs
extern "https://github.com/todaymare/margarine-std" as std;
use std::*;


fn main() {
    loop { // !!
        println("Guess the number:");
        println("Please input your guess.");

        var guess = io::read_line()!;

        print("You guessed: ");
        println(guess);
    } // !!
}

```

The `loop` keyword just loops forever. Basically identical to `while true` in almost all languages.

Next, we can add some randomization so the player has something to guess rather than just putting random stuff. The margarine standard library provides a `rand` module we can use.