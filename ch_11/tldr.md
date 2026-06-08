# 11. Writing Automated Tests

    11.1. How to Write Tests
    11.2. Controlling How Tests Are Run
    11.3. Test Organization

---

(how do i skip this 😭😭)

- i wont write this in depth

---

> Testing is a complex skill: Although we can’t cover in one chapter every detail about how to write good tests, in this chapter we will discuss the mechanics of Rust’s testing facilities. We’ll talk about the annotations and macros available to you when writing your tests, the default behavior and options provided for running your tests, and how to organize tests into unit tests and integration tests.

---

## How to Write Tests

the bodies of test functions typically perform these three actions:

- setup any needed data or state
- run the code you want to test
- assert that the results are what you expect

### Structuring Test Functions

- At its simplest, a test in Rust is a function that’s annotated with the `test` attribute
- To change a function into a test function, add `#[test]` on the line before `fn`
- When you run your tests with the cargo test command, Rust builds a test runner binary that runs the annotated functions and reports on whether each test function passes or fails.

- whenever we create a new lib fn in cargo, a test module with a test fn is automatically generated for us

- create a new lib fn called adder

```rust
$ cargo new adder --lib
     Created library `adder` project
$ cd adder
```

### Checking Results with `assert!`

The assert! macro, provided by the standard library, is useful when you want to ensure that some condition in a test evaluates to true. We give the assert! macro an argument that evaluates to a Boolean. If the value is true, nothing happens and the test passes. If the value is false, the assert! macro calls panic! to cause the test to fail.

### Testing Equality with assert_eq! and assert_ne!

A common way to verify functionality is to test for equality between the result of the code under test and the value you expect the code to return. You could do this by using the assert! macro and passing it an expression using the == operator. However, this is such a common test that the standard library provides a pair of macros—assert_eq! and assert_ne!—to perform this test more conveniently. These macros compare two arguments for equality or inequality, respectively. They’ll also print the two values if the assertion fails, which makes it easier to see why the test failed; conversely, the assert! macro only indicates that it got a false value for the == expression, without printing the values that led to the false value.

- adding custom error messages
- Checking for Panics with `should_panic`
- result

---

## Controlling How Tests are Run

bruv just read it there man :d
