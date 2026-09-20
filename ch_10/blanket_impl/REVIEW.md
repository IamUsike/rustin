# Code review: blanket impl + extension trait

**Verdict:** the first half (`Describe`) is correct. The second half (`IterDescribe`) does not compile. The compile errors come from two real gaps in how trait bounds and `IntoIterator` work, and both are worth understanding. All fixes below were compiled and run.

`cargo run` today gives 4 errors:

```
error[E0277]: `T` doesn't implement `Debug` / `Display`      (src/main.rs:20, :36)
error[E0276]: impl has stricter requirements than trait        (src/main.rs:28)
error[E0277]: `&T` is not an iterator                          (src/main.rs:42)
error[E0658]: use of unstable library feature `string_into_chars` (src/main.rs:68)
```

---

## What you got right

1. **The `Describe` blanket impl is correct.** `impl<T: Display + Debug> Describe for T` is exactly the pattern from the task, and it is the same shape as `impl<T: Display> ToString for T` in std.
2. **You interpreted the `Vec<i32>` case correctly.** `Vec<i32>` is `Debug` but not `Display`, so it does not get `describe()`. Commenting it out with the right reason shows you understood the trait-bound rules. (The task text says to "verify it on Vec<i32>". The correct verification is that it does not compile, and you found that.)
3. **You spotted the key idea in the second trait.** Your comment says "each element of the iterator implements describe, not the whole iterator itself". That is the whole point of `T::Item: Describe`.
4. **Good instinct to put the bound on `T::Item`** instead of trying to make `T` itself `Describe`.
5. **Your question in the comment has a good answer: no.** `describe()` should return the `String` and let the caller decide whether to print. Returning is more flexible (log it, test it, send it over the network). Printing inside would be a side effect that cannot be undone. Std does the same with `to_string()`.
6. **Tests-by-`main`:** you exercised `i32` and `String` and printed the results. That is a fine feedback loop at this stage.

---

## What is wrong

### 1. Missing space in the required output format (minor)

The task asks for `"display: {self}, debug: {self:?}"`. You wrote `debug:{self:?}` without the space.

```rust
// yours
format!("display: {self}, debug:{self:?}")

// correct
format!("display: {self}, debug: {self:?}")
```

### 2. Extra `where` clause on the impl method that is not on the trait (does not compile)

You put `where T::Item: Describe` on the *method in the impl*, but the trait declaration has no such bound. An impl cannot be stricter than the trait it implements. That gives E0276 and the cascade of E0277s.

```rust
// yours: bound only on the impl's method
trait IterDescribe {
    fn describe_all(&self) -> Vec<String>;
}
impl<T: IntoIterator> IterDescribe for T {
    fn describe_all(&self) -> Vec<String>
    where
        T::Item: Describe,   // <- trait does not know about this
    { ... }
}
```

The bound must be on the **impl**, so the trait stays simple and only types that qualify get the method:

```rust
impl<T> IterDescribe for T
where
    T: IntoIterator,
    T::Item: Describe,
{ ... }
```

Rule of thumb: bounds that decide *who gets the trait* go on the `impl`. Bounds that only affect one call go on the trait method itself.

### 3. `IntoIterator::into_iter` takes `self` by value, but you have `&self` (does not compile)

This is the real conceptual issue. `for res in self` with `self: &T` desugars to `IntoIterator::into_iter(self)`, which needs `&T: IntoIterator`. But you only required `T: IntoIterator`. So `&T` is not known to be iterable (E0277 "`&T` is not an iterator").

There are two clean fixes. Pick based on what you want the trait to mean.

**Option A: take `self` by value (simplest, closest to your bound).**
It consumes the collection. That is fine for iterators like `Chars`, which are one-shot anyway.

```rust
trait IterDescribe {
    fn describe_all(self) -> Vec<String>;
}

impl<T> IterDescribe for T
where
    T: IntoIterator,
    T::Item: Describe,
{
    fn describe_all(self) -> Vec<String> {
        self.into_iter().map(|x| x.describe()).collect()
    }
}
```

Tested: `vec![1, 2, 3].describe_all()` and `"ab".chars().describe_all()` both work.
Downside: `v.describe_all()` moves `v`.

**Option B: keep `&self` (matches the task signature exactly).**
Use a higher-ranked bound: "for any borrow `&'a T`, that borrow is iterable and its items are `Describe`".

```rust
trait IterDescribe {
    fn describe_all(&self) -> Vec<String>;
}

impl<T> IterDescribe for T
where
    for<'a> &'a T: IntoIterator,
    for<'a> <&'a T as IntoIterator>::Item: Describe,
{
    fn describe_all(&self) -> Vec<String> {
        self.into_iter().map(|x| x.describe()).collect()
    }
}
```

Tested: `v.describe_all()` can be called twice on the same `Vec<i32>`, and it works on `[Vec2; 1]` too. This works because `&i32` is `Display + Debug`, so `&i32: Describe` through your first blanket impl. That is blanket impls composing, which is the lesson of the exercise.
Downside: iterators such as `Chars` are not iterable by reference, so this version does not apply to them.

The task says `&self`, so Option B is the "official" answer. Option A is what most people would write in real code.

### 4. `String::into_chars()` is nightly-only (does not compile on stable)

```rust
// yours
let s_iter = s_iter.into_chars();   // E0658: unstable feature `string_into_chars`

// stable
let s_iter = "himynameischikachikaslimshady".chars();
```

`chars()` borrows the string, which is fine here. If you needed an owned iterator, `s.chars().collect::<Vec<_>>()` works, and `Vec<char>` is then iterable.

### 5. `Vec2` was skipped

"Too much work" is understandable, but it is about 8 lines and it is the case that shows the blanket impl working on *your own* type:

```rust
#[derive(Debug)]
struct Vec2 { x: i32, y: i32 }

impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// now this works with no `impl Describe for Vec2`:
Vec2 { x: 1, y: 2 }.describe()
// "display: (1, 2), debug: Vec2 { x: 1, y: 2 }"
```

This is the payoff of a blanket impl: you never write `impl Describe for Vec2`. Deriving `Debug` and implementing `Display` is enough.

---

## Smaller polish

| Item | Yours | Better |
|---|---|---|
| Redundant import | `use std::iter::IntoIterator;` | Delete. `IntoIterator` is in the prelude. |
| Manual loop | `let mut v = Vec::new(); for .. { v.push(..) }` | `.map(..).collect()` |
| `?Sized` | `impl<T: Display + Debug>` | `impl<T: Display + Debug + ?Sized>` so `str` itself is `Describe`. Optional. |
| Stray commentary in the code | Bare `//` notes inside a `where`/fn signature | Put doc comments (`///`) on the trait. |
| Naming | `res_iter` holds a `Vec<String>`, `res` is an item | `out` / `item` |
| Output | `println!("{:?}", s_vec)` | `println!("{s_vec:?}")`, matches the inline-format style you use elsewhere. |

The manual loop vs iterator chain:

```rust
// yours
let mut res_iter: Vec<String> = Vec::new();
for res in self {
    res_iter.push(res.describe());
}
res_iter

// idiomatic
self.into_iter().map(|x| x.describe()).collect()
```

---

## Suggested final solution (compiles, all task cases verified)

```rust
use std::fmt::{Debug, Display};

trait Describe {
    fn describe(&self) -> String;
}

impl<T: Display + Debug + ?Sized> Describe for T {
    fn describe(&self) -> String {
        format!("display: {self}, debug: {self:?}")
    }
}

trait IterDescribe {
    fn describe_all(&self) -> Vec<String>;
}

impl<T> IterDescribe for T
where
    for<'a> &'a T: IntoIterator,
    for<'a> <&'a T as IntoIterator>::Item: Describe,
{
    fn describe_all(&self) -> Vec<String> {
        self.into_iter().map(|x| x.describe()).collect()
    }
}

#[derive(Debug)]
struct Vec2 { x: i32, y: i32 }

impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn main() {
    println!("{}", 1.describe());
    println!("{}", String::from("hi").describe());
    println!("{}", Vec2 { x: 1, y: 2 }.describe());

    let v = vec![1, 2, 3];
    println!("{:?}", v.describe_all());
    println!("{:?}", [Vec2 { x: 1, y: 2 }].describe_all());
    // v.describe() would not compile: Vec<i32> is not Display. That is the point.
}
```

Output:

```
display: 1, debug: 1
display: hi, debug: "hi"
display: (1, 2), debug: Vec2 { x: 1, y: 2 }
["display: 1, debug: 1", "display: 2, debug: 2", "display: 3, debug: 3"]
["display: (1, 2), debug: Vec2 { x: 1, y: 2 }"]
```

---

## Takeaways to keep

1. **Bounds on an impl decide who gets the trait.** Bounds on a method that the trait does not declare are an error.
2. **`IntoIterator` is implemented for `T`, `&T` and `&mut T` separately.** With `&self` you must ask for `&T: IntoIterator`, which needs `for<'a>`. With `self` you only need `T: IntoIterator`.
3. **Blanket impls compose.** `&i32: Describe` came for free from the first blanket impl. That is why `Vec<i32>::describe_all()` works with no extra code.
4. **Return values, do not print.** Let the caller choose what to do with the string.
5. **Do not rely on nightly APIs** (`into_chars`) in exercises. Check the docs for the `unstable` badge.
