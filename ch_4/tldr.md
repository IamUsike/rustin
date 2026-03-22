4. Understanding Ownership
   4.1 What is ownership?
   4.2 References and Borrowing
   4.3 The Slice type

---

> eff. From here on, the chapters get very big so not writing too much here... ima shift back to obsidian

## What is ownership?

- It is a set of rules that govern how a rust program manages memory.
- rust manages memory through a system of ownership (set of rules that the compiler checks)
- if any of the rules are violated the program wont compile.

- all data stored in stack must have a known fixed size, else stored in heap.

### Ownership Rules

- each value in rust has an owner.
- there can only be one owner at a time.
- when an owner goes out of scope, the value will be dropped.

### Variable Scope

```rust
 {                      // s is not valid here, since it's not yet declared
        let s = "hello";   // s is valid from this point forward

        // do stuff with s
    }                      // this scope is now over, and s is no longer valid
```

### The String Type

(non-ownership aspects of string in ch-8)

- String literals (dift from strings) are immutable and are known at compile time. These are stored in stack.
- Strings are stored on the heap.

- creating a _string_ from a _string literal_

```rust
let s = String::from("hello");
```

-> this kind of string can be mutated

```rust
    let mut s = String::from("hello");

    s.push_str(", world!"); // push_str() appends a literal to a String

    println!("{s}"); // this will print `hello, world!`
```

### Memory and Allocation

- the memory for strings is dynamically allocated during runtime (on the heap)
- once we're done with the string the memory needs to be returned to the allocator.

-> Languages with garbage collector have a gc tracking free memory or in Languages like c we need to manually do this.
-> In rust, the memory is automatically returned once the variable that owns it goes out of scope.
-> when **s** goes out of scope, rust calls the `drop` function and it’s where the author of String can put the code to return the memory. Rust calls drop automatically at the closing curly bracket.

#### Variables and Data Interacting with Move

- in cases like this

```rust
    let x = 5;
    let y = x;
```

-> bind 5 to 'x' and then copy '5' to y.

- In case of string

```rust
    let s1 = String::from("hello");
    let s2 = s1;
```

- here, s1 stores `ptr, len, capacity`. `ptr` points to the memory at heap. `len` is the current occupied memory space, `capacity` is the max capacity allocated on heap.
- so, s2 will now store the ptr, len and capacity. (whole heap data wont be copied, cos expensive | shallow copy).

but in rust, once the element goes out of scope it needs to be dropped. and s1 and s2 point to the same memory, how will that work?
-> when we do s2 = s1, s1 will no longer be valid (_each value can have only one owner_)

> If you’ve heard the terms shallow copy and deep copy while working with other languages, the concept of copying the pointer, length, and capacity without copying the data probably sounds like making a shallow copy. But because Rust also invalidates the first variable, instead of being called a shallow copy, it’s known as a move. In this example, we would say that s1 was moved into s2.

**In addition, there’s a design choice that’s implied by this: Rust will never automatically create “deep” copies of your data. Therefore, any automatic copying can be assumed to be inexpensive in terms of runtime performance.**

#### Variables and data interacting with clone

- to create deep copies, we can use clone.

```rust
    let s1 = String::from("hello");
    let s2 = s1.clone();

    println!("s1 = {s1}, s2 = {s2}");
```

- simple scalar values implement the copy trait.

#### Ownership and functions

The mechanics of passing a value to a function are similar to those when assigning a value to a variable. Passing a variable to a function will move or copy, just as assignment does.

```rust
fn main() {
    let s = String::from("hello");  // s comes into scope

    takes_ownership(s);             // s's value moves into the function...
                                    // ... and so is no longer valid here

    let x = 5;                      // x comes into scope

    makes_copy(x);                  // Because i32 implements the Copy trait,
                                    // x does NOT move into the function,
                                    // so it's okay to use x afterward.

} // Here, x goes out of scope, then s. However, because s's value was moved,
  // nothing special happens.

fn takes_ownership(some_string: String) { // some_string comes into scope
    println!("{some_string}");
} // Here, some_string goes out of scope and `drop` is called. The backing
  // memory is freed.

fn makes_copy(some_integer: i32) { // some_integer comes into scope
    println!("{some_integer}");
} // Here, some_integer goes out of scope. Nothing special happens.
```

#### Return values and Scope

Returning values can also transfer ownership.

```rust
fn main() {
    let s1 = gives_ownership();        // gives_ownership moves its return
                                       // value into s1

    let s2 = String::from("hello");    // s2 comes into scope

    let s3 = takes_and_gives_back(s2); // s2 is moved into
                                       // takes_and_gives_back, which also
                                       // moves its return value into s3
} // Here, s3 goes out of scope and is dropped. s2 was moved, so nothing
  // happens. s1 goes out of scope and is dropped.

fn gives_ownership() -> String {       // gives_ownership will move its
                                       // return value into the function
                                       // that calls it

    let some_string = String::from("yours"); // some_string comes into scope

    some_string                        // some_string is returned and
                                       // moves out to the calling
                                       // function
}

// This function takes a String and returns a String.
fn takes_and_gives_back(a_string: String) -> String {
    // a_string comes into
    // scope

    a_string  // a_string is returned and moves out to the calling function
}
```

---

## References and Borrowing

- A reference is like a pointer in that it’s an address we can follow to access the data stored at that address; that data is owned by some other variable. Unlike a pointer, a reference is guaranteed to point to a valid value of a particular type for the life of that reference

```rust
fn main() {
    let s1 = String::from("hello");

    let len = calculate_length(&s1);

    println!("The length of '{s1}' is {len}.");
}

fn calculate_length(s: &String) -> usize {//value passed without   transferring the ownership
    s.len()
}
```

```rust
fn calculate_length(s: &String) -> usize { // s is a reference to a String
    s.len()
} // Here, s goes out of scope. But because s does not have ownership of what
  // it refers to, the String is not dropped.
```

> We call the action of creating a reference borrowing. As in real life, if a person owns something, you can borrow it from them. When you’re done, you have to give it back. You don’t own it.

we cant modify what we dont have ownership of.

```rust
fn main() {
    let s = String::from("hello");

    change(&s);
}

fn change(some_string: &String) {
    some_string.push_str(", world");
}
```

-> the above snippet throws a compile time error.

### Mutable references

We can fix the code from Listing 4-6 to allow us to modify a borrowed value with just a few small tweaks that use, instead, a mutable reference:

```rust
fn main() {
    let mut s = String::from("hello");

    change(&mut s);
}

fn change(some_string: &mut String) {
    some_string.push_str(", world");
}
```

Mutable references have one big restriction: If you have a mutable reference to a value, you can have no other references to that value. This code that attempts to create two mutable references to s will fail:

```rust
    let mut s = String::from("hello");

    let r1 = &mut s;
    let r2 = &mut s;

    println!("{r1}, {r2}");
```

-> the above snippet will throw a compile time error, cos we cant borrow s as mutable more than once. This prevents _data race_.

A data race is similar to a race condition and happens when these three behaviors occur: - Two or more pointers access the same data at the same time. - At least one of the pointers is being used to write to the data. - There’s no mechanism being used to synchronize access to the data.

As always, we can use curly brackets to create a new scope, allowing for multiple mutable references, just not simultaneous ones:

```rust
    let mut s = String::from("hello");

    {
        let r1 = &mut s;
    } // r1 goes out of scope here, so we can make a new reference with no problems.

    let r2 = &mut s;
```

Rust enforces a similar rule for combining mutable and immutable references. This code results in an error:

```rust
    let mut s = String::from("hello");

    let r1 = &s; // no problem
    let r2 = &s; // no problem
    let r3 = &mut s; // BIG PROBLEM

    println!("{r1}, {r2}, and {r3}");
```

- Note that a reference’s scope starts from where it is introduced and continues through the last time that reference is used. For instance, this code will compile because the last usage of the immutable references is in the println!, before the mutable reference is introduced:

```rust
    let mut s = String::from("hello");

    let r1 = &s; // no problem
    let r2 = &s; // no problem
    println!("{r1} and {r2}");
    // Variables r1 and r2 will not be used after this point.

    let r3 = &mut s; // no problem
    println!("{r3}");
```

### Dangling references

Dangling pointers reference a location in memory that may have been given to someone else by freeing the memory while preserving the pointer to that memory.

- throws compile time error

```rust
fn dangle() -> &String { // dangle returns a reference to a String

    let s = String::from("hello"); // s is a new String

    &s // we return a reference to the String, s
} // Here, s goes out of scope and is dropped, so its memory goes away.
  // Danger!
```

- the solution is to return the string directly.

---

## The slice type

Slices let you reference a contiguous sequence of elements in a collection. A slice is a kind of reference, so it does not have ownership.

```rust
    let s = String::from("hello world");

    let hello = &s[0..5];
    let world = &s[6..11];
```

- We create slices using a range within square brackets by specifying [starting_index..ending_index], where starting_index is the first position in the slice.

- the starting or the trailing number in index specifying can dropped if first or last is included respectively.

return first word

```rust
fn first_word(s: &String) -> &str {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }

    &s[..]
}
```

#### String slices as parameters

Knowing that you can take slices of literals and String values leads us to one more improvement on first_word, and that’s its signature:

```rust
fn first_word(s: &String) -> &str {
```

A more experienced Rustacean would write the signature shown in Listing 4-9 instead because it allows us to use the same function on both &String values and &str values.

```rust
fn first_word(s: &str) -> &str {
```

If we have a string slice, we can pass that directly. If we have a String, we can pass a slice of the String or a reference to the String. This flexibility takes advantage of deref coercions, a feature we will cover in the “Using Deref Coercions in Functions and Methods” section of Chapter 15.

Defining a function to take a string slice instead of a reference to a String makes our API more general and useful without losing any functionality:

```rust
fn main() {
    let my_string = String::from("hello world");

    // `first_word` works on slices of `String`s, whether partial or whole.
    let word = first_word(&my_string[0..6]);
    let word = first_word(&my_string[..]);
    // `first_word` also works on references to `String`s, which are equivalent
    // to whole slices of `String`s.
    let word = first_word(&my_string);

    let my_string_literal = "hello world";

    // `first_word` works on slices of string literals, whether partial or
    // whole.
    let word = first_word(&my_string_literal[0..6]);
    let word = first_word(&my_string_literal[..]);

    // Because string literals *are* string slices already,
    // this works too, without the slice syntax!
    let word = first_word(my_string_literal);
}
```

#### Other slices

```
let a = [1, 2, 3, 4, 5];

let slice = &a[1..3];

assert_eq!(slice, &[2, 3]);

```
