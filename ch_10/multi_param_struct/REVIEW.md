# Code Review: `StrParser<'i, 's>`

Verdict: the core lifetime idea is correct. The program compiles, runs, and shows what the exercise asks for. The problems are in `remaining()`, which can panic or return wrong data, and in the demo, which could make its point more clearly.

All snippets below were compiled with `rustc --edition 2024` to confirm the behavior described.

---

## What you got right

### 1. Two independent lifetime params
```rust
struct StrParser<'i, 's> {
    input: &'i str,
    schema: &'s str,
}
```
This is the main point of the exercise. `input` and `schema` can come from borrows that end at different times.

### 2. `remaining()` returns `&'i str`, not `&str`
```rust
fn remaining(&self) -> &'i str
```
This is the most important line in the file. The explicit `'i` ties the output to the input string only. It does not tie the output to `&self`, and it does not tie it to `'s`.

Without the annotation, lifetime elision ties the output to `&self`. Because `&self` borrows a `StrParser<'i, 's>`, the result also keeps `schema` borrowed, and your `main` stops compiling:

```rust
fn remaining(&self) -> &str { ... }   // elided = tied to &self
```
```
error[E0505]: cannot move out of `schema` because it is borrowed
```

### 3. `schema` is an owned `String`
If `schema` were a literal (`"hi"`), it would be `&'static str` and could never go out of scope, so the demo would prove nothing. Making it a `String` that you can move or drop is correct.

### 4. You understood why `parser` can't be used after the move
The commented-out `println!("{:?}", parser)` and its note are correct. `parser` holds `&'s str`, so it is dead once `schema` moves. `rem` is still valid because it only holds `'i`.

### 5. `matches()` uses `starts_with`
This is the stdlib method, and it is the right choice.

---

## What's wrong or could be better

### 1. `remaining()` can panic or return garbage (bug)
Your comment says `//always assuming that the match exists`, but nothing enforces it. `matches()` and `remaining()` are separate calls, so a caller can call `remaining()` when there is no match:

- **Wrong result:** `input = "hello"`, `schema = "xy"`, and `remaining()` returns `"llo"`. This is a silent wrong answer.
- **Panic (out of bounds):** the schema is longer than the input.
- **Panic (char boundary):** `input = "héllo"`, `schema = "ab"`, and byte index 2 lands inside `é`:
  ```
  thread 'main' panicked at ... byte index 2 is not a char boundary
  ```

**Fix:** use `str::strip_prefix`. It checks the prefix and slices safely in one step, and it keeps the `'i` lifetime:

```rust
// before
fn remaining(&self) -> &'i str {
    &self.input[self.schema.len()..]
}

// after
fn remaining(&self) -> Option<&'i str> {
    self.input.strip_prefix(self.schema)
}
```

If you must keep the exact `-> &'i str` signature from the README, choose an explicit fallback instead of slicing blindly:

```rust
fn remaining(&self) -> &'i str {
    self.input.strip_prefix(self.schema).unwrap_or(self.input)
}
```

Invalid states now can't happen, and `matches()` could be written as `self.remaining().is_some()`.

### 2. The demo is weaker than it could be
The README says: *"let the schema **go out of scope** while remaining() is still valid"*. Moving into a function works, because a move ends the borrow just like a drop. But the strongest proof is a real scope: `rem` is declared outside a block and set inside it, then used after the block closes. This is the classic lifetime demo, and it makes clear that the value **outlives** `schema` and `parser`, not only a move.

Also, `input` is a string literal (`&'static str`), so `'i` is `'static` and never under pressure. If you make `input` a `String` too, both lifetimes are real, local, and different.

```rust
fn main() {
    let input = String::from("hi my name is slim shady");  // 'i: lives the whole of main
    let rem;
    {
        let schema = String::from("hi");                    // 's: only this block
        let parser = StrParser::new(&input, &schema);
        assert!(parser.matches());
        rem = parser.remaining();
    } // schema and parser dropped here

    println!("{rem:?}"); // still valid: rem borrows only from input
}
```

### 3. Show the version that fails (the "what this cements" part)
The README says: *"if both were `'a` you couldn't express this."* Show that it fails, not only that your version works. Put this next to the demo, commented out, with the error:

```rust
struct StrParserOne<'a> {
    input: &'a str,
    schema: &'a str,
}
// Same main as above, with StrParserOne:
// error[E0597]: `schema` does not live long enough
```

With one `'a`, the compiler must pick a single lifetime that both borrows satisfy. That lifetime is the shorter one, `schema`'s, so `rem` can't escape the block.

### 4. `takeout_schema` is unnecessary
Its only job is to take ownership and drop the value. The stdlib already has this:

```rust
// before
fn takeout_schema(a: String) { println!("tookout {a}"); }
takeout_schema(schema);

// after
drop(schema);
```

### 5. Minor
- Typo in a comment: `rthe`. Also, "input(field) ka lifetime" mixes languages. Fine for personal notes, but "the input field's lifetime" is clearer to other readers.
- `#[derive(Debug)]` is only used by commented-out code. Harmless; keep it if you plan to print the parser.
- `let matches = parser.matches(); println!("{matches}");` then calls `remaining()` whatever the result was. If you keep the `&'i str` signature, guard it with `if parser.matches() { ... }`.

---

## Suggested final version

```rust
// 'i = input lifetime, 's = schema lifetime
#[derive(Debug)]
struct StrParser<'i, 's> {
    input: &'i str,
    schema: &'s str,
}

impl<'i, 's> StrParser<'i, 's> {
    fn new(input: &'i str, schema: &'s str) -> Self {
        Self { input, schema }
    }

    fn matches(&self) -> bool {
        self.input.starts_with(self.schema)
    }

    // Output borrows from input only ('i), never from schema or &self.
    fn remaining(&self) -> Option<&'i str> {
        self.input.strip_prefix(self.schema)
    }
}

fn main() {
    let input = String::from("hi my name is slim shady");
    let rem;
    {
        let schema = String::from("hi");
        let parser = StrParser::new(&input, &schema);
        assert!(parser.matches());
        rem = parser.remaining();
    } // schema and parser are gone here

    assert_eq!(rem, Some(" my name is slim shady"));
    // No match and a non-ASCII input: no panic, just None.
    assert_eq!(StrParser::new("héllo", "ab").remaining(), None);
    println!("{rem:?}");
}
```

---

## Summary

| Area | Status |
|---|---|
| Two lifetime params | Correct |
| `remaining() -> &'i str` annotation | Correct, and it is the key insight |
| Owned `schema` for the demo | Correct |
| Slicing by `schema.len()` | Bug: can panic or return a wrong result. Use `strip_prefix` |
| Scope demo | Works; a block scope with a `String` input is a clearer proof |
| Failing single-`'a` example | Missing; add it to show why two params matter |
| `takeout_schema` helper | Replace with `drop()` |
