# Code Review: `impl Trait` vs `dyn Trait`

Reviewed: `src/main.rs`  
All "better code" snippets below were compiled and run (edition 2024).

## Verdict

**Does not compile.** One error (`E0277`), at `main.rs:119`. Core idea (trait objects in `Vec<Box<dyn Drawable>>`, borrowing a `&dyn` out of the slice) is right. Several correctness and design problems sit around it.

---

## What you got right

1. **`largest_shape` signature and lifetime.** `fn largest_shape(shapes: &[Box<dyn Drawable>]) -> &dyn Drawable` matches the README. Lifetime elision ties the return to `shapes`. Correct.
2. **Reborrow from `Box`.** `&*shapes[0]` and `&**shape` are correct. Your comment (`&Box<dyn Drawable>` → `**` → `dyn Drawable`) shows you understand the deref chain.
3. **Storing mixed types.** `Vec<Box<dyn Drawable>>` is the right container for a heterogeneous collection.
4. **Conceptual notes on static vs dynamic dispatch.** "Concrete type known at compile time → monomorphization; `dyn` when the type is not known at compile time" is correct.
5. **Initialising `largest_sh`.** You noticed Rust rejects a possibly-uninitialised binding. Good instinct.
6. **`&dyn Drawable` param in `render_dynamic`.** Borrowing, not owning. Correct.
7. **Trait + three impls.** Structure is clean and easy to read.

---

## What is wrong

### 1. (Blocker) Compile error: `dyn Drawable` is not `Debug`

`main.rs:119`

```rust
println!("fin largest: {:?}", largest);
// error[E0277]: `dyn Drawable` doesn't implement `Debug`
```

`#[derive(Debug)]` on `Circle` etc. does **not** make `dyn Drawable` `Debug`. The trait object only exposes methods declared on the trait (and its supertraits). Your comment "smh idk how else to print it" points at this.

Fix A: supertrait. Every `Drawable` must be `Debug`, so `dyn Drawable` is too.

```rust
use std::fmt::Debug;

trait Drawable: Debug {
    fn draw(&self) -> String;
    fn area(&self) -> f64;
}
```

Fix B: use the trait's own method. No new bound needed.

```rust
println!("largest: {}", largest.draw());
```

Fix B is closer to the intent of the exercise (`draw` exists for this).

Also: the `use std::fmt::Debug;` you wrote is unused today. `#[derive(Debug)]` does not need it. It only becomes useful with Fix A.

### 2. (Blocker for the exercise) The `impl Trait` failure is never demonstrated

README: *"Try writing `largest_shape` with `impl Trait` instead — watch it fail. Understand why."*  
Nothing in the file attempts this, and no note explains the result. This is the point of the exercise.

Minimal demo (compiles to an error, which is the lesson):

```rust
fn pick(flag: bool) -> impl Drawable {
    if flag { Circle { radius: 1.0 } } else { Square { edge: 1.0 } }
}
// error[E0308]: `if` and `else` have incompatible types
//   expected `Circle`, found `Square`
```

Why: `-> impl Trait` means "one **specific** concrete type, chosen by the function body, hidden from the caller". The compiler monomorphizes: it needs one size and one set of methods at compile time. `if`/`else` (or `match`) returning `Circle` in one branch and `Square` in the other gives two types. Fail.

The same applies to `largest_shape`: which shape wins is decided at **runtime** (by area), so the return type cannot be fixed at compile time. Only `&dyn Drawable` / `Box<dyn Drawable>` works, because the vtable pointer carries the runtime type.

Also note: `shapes: &[impl Drawable]` would compile but is a slice of **one** type, so it cannot hold Circle + Square + Triangle. Two separate limits of `impl Trait`: return position (one type) and collection element (one type).

### 3. `largest_shape` panics on an empty slice

`main.rs:91`: `&*shapes[0]` → index out of bounds if `shapes` is empty.

Return `Option`, and let the caller decide.

### 4. `largest_shape` logic is fragile and does extra work

```rust
// yours
let mut cur_max = 0.0;
let mut largest_sh: &dyn Drawable = &*shapes[0];
for shape in shapes {
    if shape.area() > cur_max {
        cur_max = shape.area();   // area() computed twice
        largest_sh = &**shape;
    }
}
largest_sh
```

Problems:
- `area()` called twice per improvement.
- `cur_max = 0.0` hard-codes an assumption that areas are positive. With `shapes[0]` as the initial pick and `cur_max` at `0.0`, they are two sources of truth that can disagree.
- `NaN` area silently loses every comparison.
- 10 lines of state for what the iterator API already does.

Better:

```rust
fn largest_shape(shapes: &[Box<dyn Drawable>]) -> Option<&dyn Drawable> {
    shapes
        .iter()
        .max_by(|a, b| a.area().total_cmp(&b.area()))
        .map(|b| b.as_ref())
}
```

- `f64` is not `Ord`, so `max_by` + `total_cmp` (total order, handles NaN) is the standard idiom. Do not use `partial_cmp().unwrap()`, it panics on NaN.
- `b.as_ref()` turns `&Box<dyn Drawable>` into `&dyn Drawable`. Cleaner than `&**`.

If you want to keep the exact README signature (no `Option`), document the precondition and use `.expect("shapes must not be empty")`. Do not leave an implicit index panic.

### 5. `Triangle::area` is wrong

`main.rs:68`

```rust
(self.base * self.height) - 1.0 //No mood to impl traits => no div
```

Area of a triangle is `base * height / 2`. Subtracting `1.0` is not half of anything, so results are wrong for every input. Division has nothing to do with "implementing traits". It is one operator.

```rust
fn area(&self) -> f64 {
    0.5 * self.base * self.height
}
```

This bug also changes the exercise outcome: with your values (`4,4`) the triangle scores 15.0, but the true value is 8.0.

### 6. `draw()` prints instead of returning, and the render functions throw the result away

```rust
fn draw(&self) -> String {
    println!("draw circle");                 // side effect
    format!("Circle of radius {}", self.radius)
}

fn render_static(shape: impl Drawable) -> String {
    shape.draw();                            // result discarded
    String::from("Shape rendered statically")   // fixed text, says nothing about the shape
}
```

- `draw` has a `String` return type, so the string **is** the drawing. Printing from inside it is a hidden side effect, and callers cannot control it.
- `render_*` ignore what `draw` returned (`unused_must_use`-style smell) and return a constant. The `String` return value then carries no information.
- `Triangle::draw` ignores its fields: `String::from("triangle")`. Circle and Square print their dimensions. Inconsistent.

Better:

```rust
impl Drawable for Circle {
    fn draw(&self) -> String { format!("Circle(r={})", self.radius) }
    fn area(&self) -> f64 { PI * self.radius * self.radius }
}
impl Drawable for Triangle {
    fn draw(&self) -> String {
        format!("Triangle(b={}, h={})", self.base, self.height)
    }
    fn area(&self) -> f64 { 0.5 * self.base * self.height }
}

fn render_static(shape: &impl Drawable) -> String {
    format!("[static] {}", shape.draw())
}
fn render_dynamic(shape: &dyn Drawable) -> String {
    format!("[dynamic] {}", shape.draw())
}
```

### 7. `render_static` signature differs from the README

README: `fn render_static(shape: &impl Drawable)`. You wrote `shape: impl Drawable` (by value). Consequence: calling `render_static(circle)` **moves** `circle`, so you can't use it afterwards. Since `draw` takes `&self`, borrowing is enough and matches `render_dynamic`, which also makes the comparison fair (same ownership, only dispatch differs).

Also: neither render function is ever called in `main`. The exercise wants you to *feel* the difference, so call both.

### 8. Approximate π

`main.rs:46`: `self.radius * 3.14 * self.radius`. Use the constant:

```rust
use std::f64::consts::PI;
fn area(&self) -> f64 { PI * self.radius * self.radius }
```

### 9. Spec drift: `Square` vs `Rectangle`

README says `Circle, Rectangle, Triangle`. You implemented `Square`. Not wrong for the concept, but `Rectangle { width, height }` matches the spec and is no more work.

### 10. Comment on `Box` is slightly off

```rust
//box is required here cos essentially circle, square and triangle are diff types and a
//slice/vec can only contain same types elements
```

Half right. Different types can't share a `Vec<T>`: true. But `Box` alone does not fix that: `Vec<Box<Circle>>` still can't hold a `Square`. Two separate reasons:

- `dyn Drawable` makes the element type **one** type ("anything that is Drawable").
- `dyn Drawable` is **unsized** (size depends on the concrete type), and a `Vec` element must have a known size. `Box` (a pointer + vtable pointer, fixed size) gives it one. `&dyn` or `Rc<dyn>` would also work.

### 11. Comment on `render_static` vs `render_dynamic` is unfinished

```rust
//dt using static or dynamic rendering would matter here? Cos in
//the simulation, we already know which concrete type ...
```

Your answer is correct: with a known concrete type, `impl Trait` is the better tool (no vtable lookup, inlinable). Finish the thought: use `dyn` when the type is only known at runtime, which is exactly when you build the `Vec<Box<dyn Drawable>>`. Calling `render_dynamic(&circle)` on a known type works too, but pays for indirection with no benefit.

---

## Minor / style

- `#[derive(Debug)]` on the structs is fine but only useful if `Debug` gets used (see #1 Fix A).
- Debug comments like `//smh idk how else to print it` and `//No mood to impl traits` are noise. Once fixed, delete them.
- Run `cargo clippy` and `cargo fmt` before committing. Clippy reports the compile error only today; after fixing #1 it will flag the unused import if you pick Fix B.
- No tests. One tiny test would have caught the triangle bug (see below).

---

## Corrected full program

Verified with `cargo run`.

```rust
use std::f64::consts::PI;
use std::fmt::Debug;

trait Drawable: Debug {
    fn draw(&self) -> String;
    fn area(&self) -> f64;
}

#[derive(Debug)]
struct Circle { radius: f64 }
#[derive(Debug)]
struct Triangle { base: f64, height: f64 }
#[derive(Debug)]
struct Rectangle { width: f64, height: f64 }

impl Drawable for Circle {
    fn draw(&self) -> String { format!("Circle(r={})", self.radius) }
    fn area(&self) -> f64 { PI * self.radius * self.radius }
}
impl Drawable for Triangle {
    fn draw(&self) -> String { format!("Triangle(b={}, h={})", self.base, self.height) }
    fn area(&self) -> f64 { 0.5 * self.base * self.height }
}
impl Drawable for Rectangle {
    fn draw(&self) -> String { format!("Rectangle({}x{})", self.width, self.height) }
    fn area(&self) -> f64 { self.width * self.height }
}

// static: monomorphized, one copy per concrete T
fn render_static(shape: &impl Drawable) -> String {
    format!("[static] {}", shape.draw())
}
// dynamic: one copy, vtable call
fn render_dynamic(shape: &dyn Drawable) -> String {
    format!("[dynamic] {}", shape.draw())
}

fn largest_shape(shapes: &[Box<dyn Drawable>]) -> Option<&dyn Drawable> {
    shapes
        .iter()
        .max_by(|a, b| a.area().total_cmp(&b.area()))
        .map(|b| b.as_ref())
}

fn main() {
    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 4.0 }),
        Box::new(Rectangle { width: 4.0, height: 4.0 }),
        Box::new(Triangle { base: 4.0, height: 4.0 }),
    ];

    println!("{}", render_static(&Circle { radius: 1.0 }));
    println!("{}", render_dynamic(shapes[1].as_ref()));
    println!("{:?}", largest_shape(&shapes)); // Some(Circle { radius: 4.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_area_is_half_base_times_height() {
        assert_eq!(Triangle { base: 4.0, height: 4.0 }.area(), 8.0);
    }

    #[test]
    fn largest_of_empty_is_none() {
        assert!(largest_shape(&[]).is_none());
    }
}
```

Output:

```
[static] Circle(r=1)
[dynamic] Rectangle(4x4)
Some(Circle { radius: 4.0 })
```

---

## Summary table

| # | Item | Severity |
|---|------|----------|
| 1 | `{:?}` on `dyn Drawable` does not compile | Blocker |
| 2 | `impl Trait` failure never demonstrated/explained | Blocker (exercise goal) |
| 3 | Panic on empty slice | Bug |
| 5 | Triangle area formula wrong | Bug |
| 4 | Hand-rolled max, double `area()`, `0.0` sentinel | Design |
| 6 | `draw` prints; `render_*` discard result, return constants | Design |
| 7 | `render_static` takes by value, README says `&impl`; neither render fn called | Spec |
| 8 | `3.14` instead of `PI` | Minor |
| 9 | `Square` instead of `Rectangle` | Minor |
| 10–11 | Comments partly inaccurate / unfinished | Minor |
