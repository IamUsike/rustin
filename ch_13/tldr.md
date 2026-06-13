# 13. Functional Language Features: Iterators and Closures

13.1 Closures
13.2 Processing a Series of Items with Iterators
13.3 Improving our I/O project (sad)
13.4 Performance in Loops vs Iterators

---

## Closures

Rust’s closures are anonymous functions you can save in a variable or pass as arguments to other functions. You can create the closure in one place and then call the closure elsewhere to evaluate it in a different context. Unlike functions, closures can capture values from the scope in which they’re defined.

### Capturing the Environment

read in [book](https://doc.rust-lang.org/book/ch13-01-closures.html#capturing-the-environment)

### Inferring and Annotating Closure Types

Closures don’t usually require you to annotate the types of the parameters or the return value like `fn` functions do.

> type annotations can be added tho

```rust
fn  add_one_v1   (x: u32) -> u32 { x + 1 }
let add_one_v2 = |x: u32| -> u32 { x + 1 };
let add_one_v3 = |x|             { x + 1 };
let add_one_v4 = |x|               x + 1  ;
```

The first line shows a function definition and the second line shows a fully annotated closure definition. In the third line, we remove the type annotations from the closure definition. In the fourth line, we remove the brackets, which are optional because the closure body has only one expression. These are all valid definitions that will produce the same behavior when they’re called. The `add_one_v3` and `add_one_v4` lines require the closures to be evaluated to be able to compile because the types will be inferred from their usage. This is similar to `let v = Vec::new();` needing either type annotations or values of some type to be inserted into the `Vec` for Rust to be able to infer the type.

- For closure definitions, the compiler will infer one concrete type for each of their parameters and for their return value.

```rust
    let example_closure = |x| x;

    let s = example_closure(String::from("hello"));
    let n = example_closure(5);
```

The compiler gives us this error

```
$ cargo run
   Compiling closure-example v0.1.0 (file:///projects/closure-example)
error[E0308]: mismatched types
 --> src/main.rs:5:29
  |
5 |     let n = example_closure(5);
  |             --------------- ^ expected `String`, found integer
  |             |
  |             arguments to this function are incorrect
  |
note: expected because the closure was earlier called with an argument of type `String`
 --> src/main.rs:4:29
  |
4 |     let s = example_closure(String::from("hello"));
  |             --------------- ^^^^^^^^^^^^^^^^^^^^^ expected because this argument is of type `String`
  |             |
  |             in this closure call
note: closure parameter defined here
 --> src/main.rs:2:28
  |
2 |     let example_closure = |x| x;
  |                            ^
help: try using a conversion method
  |
5 |     let n = example_closure(5.to_string());
  |                              ++++++++++++

For more information about this error, try `rustc --explain E0308`.
error: could not compile `closure-example` (bin "closure-example") due to 1 previous error
```

The first time we call `example_closure` with the String value, the compiler infers the type of x and the return type of the closure to be String. Those types are then locked into the closure in `example_closure`, and we get a type error when we next try to use a different type with the same closure.

### Capturing References or Moving Ownership

Closures can capture values from their environment in three ways, which directly map to the three ways a function can take a parameter: borrowing immutably, borrowing mutably, and taking ownership. The closure will decide which of these to use based on what the body of the function does with the captured values.

- below example shows immutable borrow

```rust
fn main() {
    let list = vec![1, 2, 3];
    println!("Before defining closure: {list:?}");

    let only_borrows = || println!("From closure: {list:?}");

    println!("Before calling closure: {list:?}");
    only_borrows();
    println!("After calling closure: {list:?}");
}
```

- This example also illustrates that a variable can bind to a closure definition, and we can later call the closure by using the variable name and parentheses as if the variable name were a function name.

-> now, since we need to add a value to the vec, we borrow mutably

```rust
fn main() {
    let mut list = vec![1, 2, 3];
    println!("Before defining closure: {list:?}");

    let mut borrows_mutably = || list.push(7);

    borrows_mutably();
    println!("After calling closure: {list:?}");
}
```

Note that there’s no longer a `println!` between the definition and the call of the `borrows_mutably` closure: When `borrows_mutably` is defined, it captures a mutable reference to `list`. We don’t use the closure again after the closure is called, so the mutable borrow ends. Between the closure definition and the closure call, an immutable borrow to print isn’t allowed, because no other borrows are allowed when there’s a mutable borrow. Try adding a `println!` there to see what error message you get!

If you want to force the closure to take ownership of the values it uses in the environment even though the body of the closure doesn’t strictly need ownership, you can use the `move` keyword before the parameter list.

This technique is mostly useful when passing a closure to a new thread to move the data so that it’s owned by the new thread. We’ll discuss threads and why you would want to use them in detail in Chapter 16 when we talk about concurrency, but for now, let’s briefly explore spawning a new thread using a closure that needs the `move` keyword. Listing 13-6 shows Listing 13-4 modified to print the vector in a new thread rather than in the main thread.

```rust
use std::thread;

fn main() {
    let list = vec![1, 2, 3];
    println!("Before defining closure: {list:?}");

    thread::spawn(move || println!("From thread: {list:?}"))
        .join()
        .unwrap();
}
```

### Moving Captured Values Out of Closures

Once a closure has captured a reference or captured ownership of a value from the environment where the closure is defined (thus affecting what, if anything, is moved _into_ the closure), the code in the body of the closure defines what happens to the references or values when the closure is evaluated later (thus affecting what, if anything, is moved _out of_ the closure).

A closure body can do any of the following: Move a captured value out of the closure, mutate the captured value, neither move nor mutate the value, or capture nothing from the environment to begin with.

The way a closure captures and handles values from the environment affects which traits the closure implements, and traits are how functions and structs can specify what kinds of closures they can use. Closures will automatically implement one, two, or all three of these `Fn` traits, in an additive fashion, depending on how the closure’s body handles the values:

- `FnOnce` applies to closures that can be called once. All closures implement at least this trait because all closures can be called. A closure that moves captured values out of its body will only implement `FnOnce` and none of the other `Fn` traits because it can only be called once.

- `FnMut` applies to closures that don’t move captured values out of their body but might mutate the captured values. These closures can be called more than once.

- `Fn` applies to closures that don’t move captured values out of their body and don’t mutate captured values, as well as closures that capture nothing from their environment. These closures can be called more than once without mutating their environment, which is important in cases such as calling a closure multiple times concurrently.

if we look at it fn is a subest of fnmut which is a subset of FnOnce

---

## Processing a Series of Items with Iterators

- The iterator pattern allows us to perform some task on a sequence of items in turn.
- An iterator is responsible for the logic of iterating over each item and determining when the sequence has finished.

- Iterators are _lazy_ in rust, meaning they have no effect until you call methods that consume the iterator to use it up.

```
    let v1 = vec![1, 2, 3];

    let v1_iter = v1.iter();

    for val in v1_iter {
        println!("Got: {val}");
    }
```

- when we use a for loop on a vec(without doing `.iter()` before). It is implicitly creates an iterator and consumes it.

### The `Iterator` Trait and the `next` Method

- All iterators implement a trait named `Iterator` thats defined in the std lib.

the definition of the trait looks like this

```rust
put trait Iterator {
  type Item;

  fn next(&mut self) -> Option<Self::Item>
}
```

Notice that this definition uses some new syntax: `type Item` and `Self::Item`, which are defining an associated type with this trait. We’ll talk about associated types in depth in Chapter 20. For now, all you need to know is that this code says implementing the `Iterator` trait requires that you also define an `Item` type, and this `Item` type is used in the return type of the `next` method. In other words, the `Item` type will be the type returned from the iterator.

The `Iterator` trait only requires implementors to define one method: the `next` method, which returns one item of the iterator at a time, wrapped in `Some`, and, when iteration is over, returns `None`.

We can call the `next` method on iterators directly; Listing 13-12 demonstrates what values are returned from repeated calls to `next` on the iterator created from the vector.

Filename: `src/lib.rs`

```rust
#[test]
fn iterator_demonstration() {
    let v1 = vec![1, 2, 3];

    let mut v1_iter = v1.iter();

    assert_eq!(v1_iter.next(), Some(&1));
    assert_eq!(v1_iter.next(), Some(&2));
    assert_eq!(v1_iter.next(), Some(&3));
    assert_eq!(v1_iter.next(), None);
}
```

Note that we needed to make v1_iter mutable: Calling the next method on an iterator changes internal state that the iterator uses to keep track of where it is in the sequence. In other words, this code consumes, or uses up, the iterator. Each call to next eats up an item from the iterator. We didn’t need to make v1_iter mutable when we used a for loop, because the loop took ownership of v1_iter and made it mutable behind the scenes.

Also note that the values we get from the calls to next are immutable references to the values in the vector. The iter method produces an iterator over immutable references. If we want to create an iterator that takes ownership of v1 and returns owned values, we can call into_iter instead of iter. Similarly, if we want to iterate over mutable references, we can call iter_mut instead of iter.

### Methods that Consume the Iterator

- Methods that call `next` are called _consuming adapters_ because calling them uses up the iterator.

### Methods That Produce Other Iterators

- Iterator adapters are methods defined on the Iterator trait that don’t consume the iterator. Instead, they produce different iterators by changing some aspect of the original iterator.

(read the book for examples if dbt)

### Closures That Capture Their Environment

Many iterator adapters take closures as arguments, and commonly the closures we’ll specify as arguments to iterator adapters will be closures that capture their environment.

---

### Performance in Loops vs. Iterators

iterators are one of rusts zero cost abstractions
