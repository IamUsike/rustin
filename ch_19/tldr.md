# Patterns and Matching

Patterns are a special syntax in Rust for matching against the structure of types, both complex and simple.

To use a pattern, we compare it to some value. If the pattern matches the value, we use the value parts in our code.

## All the places patterns can be used

### Match Arms

```rust
match VALUE {
    PATTERN => EXPRESSION,
    PATTERN => EXPRESSION,
    PATTERN => EXPRESSION,
}
```

### `let` statements

```rust
let PATTERN = EXPRESSION;
```

```rust
let (x, y, z) = (1, 2, 3);
```

### Conditional `if let` statements

```rust
fn main() {
    let favorite_color: Option<&str> = None;
    let is_tuesday = false;
    let age: Result<u8, _> = "34".parse();

    if let Some(color) = favorite_color {
        println!("Using your favorite color, {color}, as the background");
    } else if is_tuesday {
        println!("Tuesday is green day!");
    } else if let Ok(age) = age {
        if age > 30 {
            println!("Using purple as the background color");
        } else {
            println!("Using orange as the background color");
        }
    } else {
        println!("Using blue as the background color");
    }
}
```

### `while let` Conditional loops

Similar in construction to if let, the while let conditional loop allows a while loop to run for as long as a pattern continues to match.

```rust
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for val in [1, 2, 3] {
            tx.send(val).unwrap();
        }
    });

    while let Ok(value) = rx.recv() {
        println!("{value}");
    }
```

### `for` loops

```rust
    let v = vec!['a', 'b', 'c'];

    for (index, value) in v.iter().enumerate() {
        println!("{value} is at index {index}");
    }
```

### Function Params

```rust
fn print_coordinates(&(x, y): &(i32, i32)) {
    println!("Current location: ({x}, {y})");
}

fn main() {
    let point = (3, 5);
    print_coordinates(&point);
}
```

---

## Refutability: Whether a Pattern Might Fail to Match

Patterns come in 2 forms:

- **irrefutable** : Patterns that match for any possible value.
- **refutable**: Patterns that can fail to match for some values.

Function parameters, let statements, and for loops can only accept irrefutable patterns because the program cannot do anything meaningful when values don’t match. The if let and while let expressions and the let...else statement accept refutable and irrefutable patterns, but the compiler warns against irrefutable patterns because, by definition, they’re intended to handle possible failure: The functionality of a conditional is in its ability to perform differently depending on success or failure.

---

### Pattern syntax

(read book)

1. matching literals

- use match

2. matching named vars
3. destrtucturing structs
