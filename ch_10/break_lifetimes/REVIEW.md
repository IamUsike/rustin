# Code Review: Break Lifetimes, 5 Ways

This review checks `src/main.rs` against the task in `README.md`. All snippets here were compiled with `rustc --edition 2024`, and each error code shown is the real compiler output.

## Summary

| # | Task | Your error | Verdict |
|---|------|------------|---------|
| 1 | Return a reference to a local | `E0106` missing lifetime specifier | Wrong error. The fixed signature would compile, so no dangling reference is shown |
| 2 | Use a reference after a move | `E0505` cannot move out of `a` | Correct |
| 3 | Two `&mut` to the same data | `E0499` | Correct. The explanation needs a small adjustment |
| 4 | Struct used after its referent is dropped | `E0505` | Correct bug. The error comes from the move, not from scope |
| 5 | `&mut self` call while a `&self` borrow is held | none (compiles) | Missing. The conflict is never created |

The README also asks for a broken version, a fix, and an explanation for each case. You wrote the broken versions and most of the explanations, but no fixes. None of the functions are called from `main`. A working version with all fixes is at the end of this file.

---

## What you got right

- **Case 2** triggers exactly the intended error. Your comment, "since there's a borrow already that value can't be moved (would lead to a dangling ref)", explains the bug correctly.
- **Case 3** triggers `E0499`. You identified reallocation as the real danger: `push` can move the buffer, which leaves the other reference pointing at freed memory. That is the key insight.
- **Case 4** uses `drop(s)` explicitly. That shows the bug clearly: `a.ref_val` would point at freed heap memory.
- The struct `Refi<'a>` has the correct lifetime parameter.
- Each case is in its own function. That is a clean layout for the exercise.

---

## Case 1: return a reference to a local

### What happens

```rust
fn re_local() -> &str {
    let l = "hey";
    &l
}
```

The compiler reports `E0106: missing lifetime specifier`. This is a *syntax/elision* error, not a borrow-check error. The borrow checker never looks at your body. The compiler only notes that a function with no reference inputs must say where the output reference comes from.

### Hidden problem

`"hey"` is a string literal. Its type is `&'static str`, and the text lives in the binary for the entire program. If you add the lifetime, the code **compiles and runs**:

```rust
fn re_local() -> &'static str {
    let l = "hey";
    &l // &&str auto-derefs to &'static str. This is fine.
}
```

The local variable `l` is only a pointer. The data it points to is never dropped. So this example cannot produce the bug that the task describes.

Your comment says "we can't really represent this in lifetimes". That is close: you cannot name a lifetime that outlives the function *for data owned by the function*. But that only applies when the function owns the data, for example a `String`.

### Correct broken version

```rust
fn re_local() -> &'static str {
    let l = String::from("hey"); // heap data OWNED by this function
    &l
}
// error[E0515]: cannot return reference to local variable `l`
```

### Fix

Return the owned value, so the caller takes ownership:

```rust
fn re_local() -> String {
    let l = String::from("hey");
    l
}
```

**What the borrow checker prevented:** `l` is freed when `re_local` returns. A returned `&str` would point at freed heap memory, which is a use-after-free.

---

## Case 2: use a reference after a move

Correct. Two small points:

- `let c = a;` gives an `unused variable` warning. Use `_c` or print it.
- A fix is missing. Finish using the borrow before the move:

```rust
fn use_ref() {
    let a = "helo".to_string();
    let b = &a;
    println!("{b}"); // last use of b, so the borrow ends here (NLL)
    let c = a;       // move is now allowed
    println!("{c}");
}
```

This fix works because of **non-lexical lifetimes (NLL)**. A borrow lasts until its last use, not until the end of the scope. You will use this in cases 3, 4 and 5 also.

---

## Case 3: two mutable references

The error is correct. Small issues:

- `let mut d1` / `let mut d2`: the `mut` is not needed and gives warnings. `d1` is already an `&mut Vec`. `mut` on the binding only lets you reassign `d1` itself.
- "Data races" is not quite the reason for this example. Your code has only one thread. The general rule is **aliasing XOR mutability**. It prevents data races in threaded code. In single-threaded code, it prevents *invalidation*: one `&mut` reallocates the buffer, and the other `&mut` still points at the old buffer. Your reallocation point is the correct explanation here.

Fix: do not let the two borrows overlap.

```rust
fn mut_ref() {
    let mut data = vec![1, 2];

    let d1 = &mut data;
    d1.push(1);          // d1's last use; borrow ends here

    let d2 = &mut data;  // now allowed
    d2.push(2);

    println!("{data:?}"); // [1, 2, 1, 2]
}
```

---

## Case 4: struct used after its referent is dropped

The bug is correct. Note the error code: `drop(s)` *moves* `s` into `drop`, so you get `E0505` (move while borrowed). The same error appears in case 2. The typical "referent dropped" error is `E0597`. It appears when the owner goes out of **scope** while the struct still exists:

```rust
fn stru() {
    let a;
    {
        let s = String::from("hjell");
        a = Refi { ref_val: &s };
    } // s dropped here
    println!("{a:?}");
}
// error[E0597]: `s` does not live long enough
```

Both versions are valid demonstrations. `E0597` shows the *lifetime* part more clearly, because `'a` must outlive the struct and `s` does not.

Your comment for the fix is correct ("drop the struct before `s` goes out of scope"). Here it is as code:

```rust
fn stru() {
    let s = "hjell".to_string();
    let a = Refi { ref_val: &s };
    println!("{a:?}"); // last use of a
    drop(s);           // fine: no live borrow of s remains
}
```

Also, `println!("{:?}", a)` can be written as `println!("{a:?}")` (inline format args).

---

## Case 5: `&mut self` call while holding `&self`

### What is wrong

```rust
impl<'a> Refi<'a> {
    fn dk(&mut self) {
        println!("hello");
    }
}
```

This only *defines* a `&mut self` method. Nothing calls it, and no `&self` borrow is held. So the code compiles and does not demonstrate the error. Your comment ("similar to the previous case") is correct in principle, but the code must actually show the conflict.

Also: `dk` does not use `'a`. You can write `impl Refi<'_>`.

### Correct broken version

Add a `&self` method that returns a reference. Hold that reference, then call the `&mut self` method:

```rust
struct Refi<'a> {
    ref_val: &'a str,
    hits: u32,
}

impl Refi<'_> {
    fn get(&self) -> &str { self.ref_val } // output tied to &self (elision rule)
    fn bump(&mut self) { self.hits += 1; }
}

fn main() {
    let s = String::from("x");
    let mut r = Refi { ref_val: &s, hits: 0 };
    let v = r.get();  // &self borrow starts, and stays live while v is live
    r.bump();         // needs &mut self
    println!("{v}");  // v still in use
}
// error[E0502]: cannot borrow `r` as mutable because it is also borrowed as immutable
```

**What the borrow checker prevented:** `bump` gets exclusive access to `r`. It could change or replace `r.ref_val`. `v` could then point at something that changed while you read it. In a type such as `Vec`, it could point at freed memory.

### Fix, and a lifetime lesson

There are two fixes. The second one is the most important lifetime concept in this exercise:

```rust
impl<'a> Refi<'a> {
    // Elided: returns &'self str. The result keeps `self` borrowed.
    fn get(&self) -> &str { self.ref_val }

    // Explicit: returns &'a str. The result is tied to the ORIGINAL string,
    // not to the struct borrow, so `self` is free as soon as the call returns.
    fn get_inner(&self) -> &'a str { self.ref_val }

    fn bump(&mut self) { self.hits += 1; }
}

fn self_borrow() {
    let s = String::from("x");
    let mut r = Refi { ref_val: &s, hits: 0 };

    // Fix A: copy out an owned value, so the borrow ends immediately.
    let v = r.get().to_string();
    r.bump();
    println!("{v}");

    // Fix B: return &'a str. This compiles and still returns a reference.
    let w = r.get_inner();
    r.bump();
    println!("{w} {}", r.hits);
}
```

Fix B works because `'a` refers to `s`, not to `r`. Mutating `r` cannot invalidate `s`. The elided signature does not give the compiler that information, so the compiler assumes the more restrictive case.

---

## Structure: meeting the README

The README asks for broken version, fix, and explanation per case. A simple layout that still compiles:

- Keep each broken version in a comment block, with the exact error code, for example `// error[E0515]: ...`.
- Put the fixed version below the comment as real code, and call it from `main`.

This way `cargo run` works, and you can still read each error. The complete fixed program, which compiles without warnings:

```rust
fn re_local() -> String {
    let l = String::from("hey");
    l
}

fn use_ref() {
    let a = "helo".to_string();
    let b = &a;
    println!("{b}");
    let c = a;
    println!("{c}");
}

fn mut_ref() {
    let mut data = vec![1, 2];
    let d1 = &mut data;
    d1.push(1);
    let d2 = &mut data;
    d2.push(2);
    println!("{data:?}");
}

#[derive(Debug)]
struct Refi<'a> {
    ref_val: &'a str,
    hits: u32,
}

impl<'a> Refi<'a> {
    fn get(&self) -> &str { self.ref_val }
    fn get_inner(&self) -> &'a str { self.ref_val }
    fn bump(&mut self) { self.hits += 1; }
}

fn stru() {
    let s = "hjell".to_string();
    let a = Refi { ref_val: &s, hits: 0 };
    println!("{a:?}");
    drop(s);
}

fn self_borrow() {
    let s = String::from("x");
    let mut r = Refi { ref_val: &s, hits: 0 };

    let v = r.get().to_string();
    r.bump();
    println!("{v}");

    let w = r.get_inner();
    r.bump();
    println!("{w} {}", r.hits);
}

fn main() {
    println!("{}", re_local());
    use_ref();
    mut_ref();
    stru();
    self_borrow();
}
```

Output:

```
hey
helo
helo
[1, 2, 1, 2]
Refi { ref_val: "hjell", hits: 0 }
x
x 2
```

## Minor

- Comment typos: "moce", "becuuse", "reallaction", "beofre", "ligetime", "exection". Not important, but correct spelling makes the notes easier to search later.
- `use_ref` uses `print!` without a newline. Use `println!`.
