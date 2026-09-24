# Code Review: Explicit Lifetimes Exercise

Scope: `src/main.rs` against the task in `README.md`.
All panic cases below were reproduced with `catch_unwind`, and every fix was checked with `assert_eq!`.

## Summary

| # | Item | Annotations | Comment | Runtime behavior |
|---|------|-------------|---------|------------------|
| 1 | `first_word` | Correct | Framing slightly off | Panics when the input has no space |
| 2 | `longer` | Correct | Mostly right, can be more precise | Correct |
| 3 | `first` | Correct | Says output is `T`, but it is `&T` | Panics on an empty slice |
| 4 | `StrSplit` | Correct | **Direction is reversed** | n/a |
| 5 | `substr` | Correct | Framing slightly off | Panics on bad range, UTF-8 boundary, overflow |

All five signatures are annotated correctly. The part of the exercise that needs work is the comments, which is where the README says the real point is.

---

## What you got right

- **Every signature is correct.** `'a` is on the right inputs and outputs everywhere, including `first_word`, where elision would normally hide it. That was the task.
- **`longer` uses one lifetime for both inputs.** This is the key idea: the compiler can't know which branch returns, so both inputs must share `'a`.
- **`substr` puts no lifetime on `start` and `len`.** Correct: `usize` is `Copy` and not borrowed, so it has no part in the relationship.
- **`first<'a, T>`** declares the lifetime before the type parameter. That is the required order.
- **`StrSplit<'a>`** declares the lifetime on the struct and uses it on the field. This is required for any struct that holds a reference.

---

## The core misconception (affects comments 1, 3, 4, 5)

Most of your comments say something like "the output **will live as long as** the input". This reads as if a lifetime annotation *controls* how long a value lives. It does not.

A lifetime annotation never extends or shortens anything. It is a **constraint** that the borrow checker verifies:

> `fn f<'a>(s: &'a str) -> &'a str` means: "the returned reference borrows from `s`, so it **can't outlive** `s`."

The output can live *shorter* than the input (it usually does). What is forbidden is for it to live *longer*. "Can't outlive" is the right phrase. "Will live as long as" is not.

---

## 1. `first_word`

```rust
//this means that the output will live for as long the the input.
fn first_word<'a>(s: &'a str) -> &'a str {
    let (first, _) = s.split_once(" ").unwrap();
    first
}
```

**Bug: panics when there is no space.** `first_word("hello")` gives:

```
called `Option::unwrap()` on a `None` value
```

A single word *is* the first word, so the function should return the whole input.

**Minor:** `" "` is a `&str` pattern. `' '` (a `char`) is the idiomatic choice for one character.

**Fix:**

```rust
// 'a ties the output to `s`: the returned slice points into `s`,
// so it can't outlive the string it was cut from.
// (Elision rule: one input reference -> output gets its lifetime.)
fn first_word<'a>(s: &'a str) -> &'a str {
    s.split(' ').next().unwrap_or(s)
}
```

`split` always yields at least one item, so `unwrap_or(s)` never triggers. It is only there to avoid `unwrap()`.

---

## 2. `longer`

```rust
//the output will live as long as both s1 and s2 exist
fn longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        return s1;
    }
    s2
}
```

**Logic is correct.** On a tie it returns `s2`. That is fine, but state it.

**Comment:** close. A more precise explanation: at the call site, the compiler picks `'a` as the **overlap** of the two borrows, which is the shorter one. The result is only usable while *both* inputs are still valid. This is also why elision can't help here: with two input references, the compiler has no rule to choose which one the output comes from.

**Style:** an early `return` works, but `if`/`else` is an expression in Rust, and that is the idiomatic form:

```rust
// Both inputs share 'a, so 'a shrinks to the shorter of the two borrows.
// The result can't outlive EITHER input, because we don't know until
// runtime which one is returned. Elision can't apply: two input refs.
// On a tie, s2 is returned.
fn longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() { s1 } else { s2 }
}
```

---

## 3. `first`

```rust
//we declare a generic lifetim 'a  and a generic param T
//The output (of type T) lives for as long as the input
fn first<'a, T>(silce: &'a [T]) -> Option<&'a T> {
    Some(&silce[0])
}
```

**Bug: panics on an empty slice, which defeats the `Option`.** `first(&[] as &[i32])` gives:

```
index out of bounds: the len is 0 but the index is 0
```

The return type promises "maybe there is no first element". The body never returns `None`, so it breaks that promise.

**Comment is inaccurate:** the output is not "of type `T`". It is `Option<&'a T>`, a *reference* to an element that lives inside the slice. That is the whole point of `'a`: the returned `&T` points into the slice's memory, so it can't outlive the slice.

**Typos:** `silce` should be `slice`, and `lifetim` should be `lifetime`.

**Fix:** the standard library already has this method.

```rust
// 'a ties the returned element reference to the slice's borrow.
// The &T points into the slice, so it can't outlive the slice.
// T itself needs no lifetime: we never own or copy a T, we only borrow one.
fn first<'a, T>(slice: &'a [T]) -> Option<&'a T> {
    slice.first() // same as slice.get(0); returns None when empty
}
```

---

## 4. `StrSplit<'a>`: the biggest issue

```rust
//the instance of this struct will be kept alive until
//the value referred to in the remainder field exists
struct StrSplit<'a> {
    remainder: &'a str,
}
```

**The comment has the relationship backwards.** It says the struct is *kept alive* by the string. In fact:

- Nothing keeps anything alive. Lifetimes don't extend lifetimes.
- The constraint goes the other way: the **struct can't outlive the string** it borrows.

The string can live much longer than the struct, which is the normal case. Here is the compiler enforcing the real rule:

```rust
let split;
{
    let s = String::from("a,b,c");
    split = StrSplit { remainder: &s };
} // `s` dropped here
println!("{}", split.remainder);
// error[E0597]: `s` does not live long enough
```

The compiler rejects this because the *struct* would outlive the *string*. It does not keep `s` alive to fix the problem.

**Fix:**

```rust
// StrSplit<'a> holds a borrowed &str, not an owned String.
// 'a says: any StrSplit instance can't outlive the string that
// `remainder` points into. Structs have no lifetime elision, so any
// field that holds a reference forces a lifetime parameter.
struct StrSplit<'a> {
    remainder: &'a str,
}
```

---

## 5. `substr`

```rust
//the output will be alive as long as the input param (s) exists
fn substr<'a>(s: &'a str, start: usize, len: usize) -> &'a str {
    &s[start..start + len]
}
```

The annotation is correct. The same "can't outlive" note applies to the comment.

**Bugs: three different panics** on input that is easy to get wrong:

| Input | Panic |
|-------|-------|
| `substr("hi", 0, 10)` | range end out of bounds |
| `substr("héllo", 1, 1)` | `byte index 2 is not a char boundary; it is inside 'é' (bytes 1..3)` |
| `substr("hi", 1, usize::MAX)` | `attempt to add with overflow` (debug build) |

The char-boundary case is the important one to learn: `str` indices are **byte** offsets, not character positions.

**Fix:** use `str::get`, which returns `None` instead of panicking. Use `checked_add` for the overflow case.

```rust
// 'a: the returned &str is a view into `s`, so it can't outlive `s`.
// `start` and `len` are plain usize values (Copy, not borrowed),
// so they take no part in the lifetime relationship.
// Indices are BYTE offsets; returns None if out of range or not on a char boundary.
fn substr<'a>(s: &'a str, start: usize, len: usize) -> Option<&'a str> {
    s.get(start..start.checked_add(len)?)
}
```

If you must keep the `-> &'a str` signature from the README, document the panics in a `# Panics` doc section instead of hiding them.

---

## Smaller notes

- **Dead code:** until `main` called them, every function triggered `dead_code` warnings. I added one call per item to `main`, as your comment asked, and nothing else.
- **Comment style:** `// text` (with a space) is the rustfmt/community convention. For item docs, `///` doc comments show up in `cargo doc` and on hover in your editor.
- **Typo:** "as long the the input" should be "as long as the input".

## Takeaway

The signatures show that you understand *where* lifetimes go. The comments show one remaining gap: seeing `'a` as a **duration** instead of a **relationship**. Replace "lives as long as" with "can't outlive" in every comment. For `StrSplit`, flip the direction: the borrower is bounded by the owner, never the other way around.
