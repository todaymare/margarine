
# Chapter 1
## Setting Up a New Project
To set up a new projects, go to any directory you'd like and just create a file. Doesn't even need to end with `.mar`, though that is preferred. I gotta brand this somehow.  

## Making a Guessing Game
Let's make a guessing game shall we?

### Processing a guess
```rs
extern "pkg:std" as std;
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

Firstly,
```rs
extern "pkg:std" as std;
```
This line pulls the "std" package from the default package system. The string can be *any* git url but the `pkg:` prefix is a shortcut for the default package system.

This package system is determined by the `MARGARINE_DEFAULT_URL` environment variable. If the variable is missing it'll default to `pkg.daymare.net/margarine`

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
extern "pkg:std" as std;
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

Before we can compare the guess with the randomly generated number, we'll need to parse it from a string to an integer.

```rs
extern "pkg:std" as std;
use std::*;


fn main() {
    loop { // !!
        println("Guess the number:");
        println("Please input your guess.");

        var guess = io::read_line()!;

        print("You guessed: ");
        println(guess);

        var guess : int = guess.parse()!;

        var computer_guess = rand::random_range(0..10);
        if computer_guess < guess {
            println("You guessed too high!");
        } else if computer_guess > guess {
            println("You guessed too low!");
        } else {
            println("Correct!");
        }
    }
}

```

The .. operator creates a range between the left (min) and right (max). The lower bound is inclusive while the upper bound is exclusive. If you want the upper bound to be inclusive as well you can use the ..= operator.