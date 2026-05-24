# Rust Lifetime Elision Rules — Examples

## Rule 1: Every reference gets its own lifetime

```rust
fn foo(x: &i32, y: &i32) {
    // compiler interprets this as:
    // fn foo<'a, 'b>(x: &'a i32, y: &'b i32)
}
```

- Each reference is independent.
- No relationship is assumed between x and y.

## Rule 2: Single input lifetime → assigned to output

```rust
fn get(x: &i32) -> &i32 {
    x
}
```

Equivalent to:

```rust
fn get<'a>(x: &'a i32) -> &'a i32
```

- Output lifetime is tied to the only input.

## Rule 3: Methods → `self` lifetime is used

```rust
struct A {
    val: i32
}

impl A {
    fn get_val(&self) -> &i32 {
        &self.val
    }
}
```

Equivalent to:

```rust
fn get_val<'a>(&'a self) -> &'a i32
```

- Return value lives as long as self.

---

## Case: Ambiguous lifetimes (Fails)

```rust
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}
```

❌ Error:
Compiler cannot determine whether return depends on x or y

FIX:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str
```

## Case: Works (Single input → Rule 2)

```rust
fn first_char(s: &str) -> &str {
    &s[0..1]
}
```

Only one input → output tied automatically

## Case: Works (Method → Rule 3)

```rust
impl String {
    fn first_char(&self) -> &str {
        &self[0..1]
    }
}
```

- self determines output lifetime

## Case: Fails (Even if returning only one param)

```rust
fn pick(x: &str, y: &str) -> &str {
    x
}
```

❌ Still fails:
Compiler doesn’t assume output is from x

FIX

```rust
fn pick<'a>(x: &'a str, y: &str) -> &'a str
```

## Case: Mixed lifetimes (Explicit control)

```rust
fn combine<'a>(x: &'a str, y: &str) -> &'a str {
    x
}
```

- Output tied only to x
- y is unrelated
