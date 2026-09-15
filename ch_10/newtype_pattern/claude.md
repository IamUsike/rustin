# Code Review — `Validated<T>` Newtype Pattern

## 1. Critical bug: `new()` doesn't actually build a `Validated<T>`

**Your code:**

```rust
fn new(val: T) -> Result<String, String> {
    match val.validate() {
        Ok(()) => Ok(String::from("all good my ni")),
        Err(err) => Err(err),
    }
}
```

**The problem:** The exercise asks for `fn new(val: T) -> Result<Self, String>`. Your version returns `Result<String, String>` — a success message — instead of the wrapped value. This defeats the entire point of the newtype pattern: `Validated<T>` is supposed to be a type that can _only_ exist in a validated state, so that once you have a `Validated<Email>`, the compiler guarantees you never have to re-check it. Right now, `new` validates the input and then **throws it away**, returning a string instead of a `Validated<T>`. Nothing produced by your `new` can ever be passed to `inner()`, because nothing of type `Validated<T>` is ever constructed by it.

**Right idea (contrast, not your full solution):**

```rust
fn new(val: T) -> Result<Self, String> {
    match val.validate() {
        Ok(()) => Ok(Validated(val)),
        Err(err) => Err(err),
    }
}
```

The key shift: on success, you wrap `val` in `Self` (i.e. `Validated(val)`), not a message. The success message printed in `main` should come from formatting _after_ construction, not from baking a string into the type itself.

This also means your `main` is only working "by accident" — it prints a message, but `valid_email` is not a `Validated<Email>` at all, it's a `String`. If you tried `valid_email.inner()` right now, it wouldn't compile, because `String` has no `inner` method.

---

## 2. Missing: `NonEmptyString`

The exercise calls for three validated types: `Email`, `Port`, and `NonEmptyString`. You've implemented `Validate` for the first two only. This one's on you to add — it's a good one to do yourself since it's the simplest of the three (just check `!self.0.is_empty()`), and doing it will confirm whether the `new`/`inner` pattern above actually clicked.

---

## 3. Missing: proof that `Validated<Email>` and `Validated<Port>` are separate types

The exercise specifically asks you to _show_ — at compile time — that `Validated<Email>` and `Validated<Port>` can't be mixed up. Nothing in your code currently demonstrates this. A common way to show it is a comment showing what _fails_ to compile, e.g. attempting to assign a `Validated<Port>` to a variable typed as `Validated<Email>`, or passing one where the other is expected, and noting the compiler error. Right now this requirement isn't addressed at all — worth adding once `new` is fixed, since it's the actual payoff of the whole exercise.

---

## 4. Your `Port` validation — good catch, and you're right

```rust
let condition = self.0 < 1;
```

Your comment questioning the upper-bound check is correct: `self.0` is a `u16`, whose max value is exactly `65535` — the same as the upper bound the spec asks for. Since a `u16` can never exceed `65535` in the first place, checking `self.0 > 65535` would always be `false` and is dead code. Good instinct catching that instead of copy-pasting a redundant check.

One nitpick, not a bug: `self.0 < 1` for an unsigned integer is equivalent to `self.0 == 0`, since `0` is the smallest value `u16` can hold. Either reads fine, but `== 0` communicates intent slightly more directly (there's no "negative port" case to guard against with an unsigned type).

---

## 5. Your `Email` validation — logically correct, but redundant

```rust
if !self.0.contains('@') || self.0.is_empty() {
    return Err(String::from("Invalid email"));
}
```

This works, but the `is_empty()` check never adds anything: an empty string can never contain `'@'`, so `!self.0.contains('@')` is already `true` whenever the string is empty. The `is_empty()` check is short-circuited into irrelevance — it never independently triggers the `Err`. Not wrong, just dead weight.

---

## 6. `inner()` — correct

```rust
fn inner(&self) -> &T {
    &self.0
}
```

No notes — this correctly borrows the wrapped value rather than moving it out, which is exactly right for an accessor method like this.

---

## Summary

| Item                                                      | Status                                                                 |
| --------------------------------------------------------- | ---------------------------------------------------------------------- |
| `impl<T: Validate> Validated<T>` bound                    | ✅ Correct                                                             |
| `Validate for Email`                                      | ✅ Correct (minor redundant check)                                     |
| `Validate for Port`                                       | ✅ Correct (good instinct on the upper bound)                          |
| `Validate for NonEmptyString`                             | ❌ Missing                                                             |
| `new()` returns `Self` on success                         | ❌ Bug — currently returns a `String`, never constructs `Validated<T>` |
| `inner()`                                                 | ✅ Correct                                                             |
| Demonstration that `Validated<Email>` ≠ `Validated<Port>` | ❌ Missing                                                             |

The core trait-bound reasoning (`T: Validate`, separate `impl` blocks per type) is solid — the main thing to fix is that `new` needs to actually _produce_ a `Validated<T>` rather than a status message, since that's the whole mechanism the newtype pattern relies on.

---

## Corrected Solution

```rust
struct Validated<T>(T);

trait Validate {
    fn validate(&self) -> Result<(), String>;
}

struct Email(String);
struct Port(u16);
struct NonEmptyString(String);

impl Validate for Email {
    fn validate(&self) -> Result<(), String> {
        if !self.0.contains('@') {
            return Err(String::from("Invalid email"));
        }
        Ok(())
    }
}

impl Validate for Port {
    fn validate(&self) -> Result<(), String> {
        // u16's max is already 65535, so no upper-bound check is needed
        if self.0 == 0 {
            return Err(String::from("invalid port"));
        }
        Ok(())
    }
}

impl Validate for NonEmptyString {
    fn validate(&self) -> Result<(), String> {
        if self.0.is_empty() {
            return Err(String::from("string must not be empty"));
        }
        Ok(())
    }
}

impl<T: Validate> Validated<T> {
    // returns Err if validate fails, otherwise wraps val in Self
    fn new(val: T) -> Result<Self, String> {
        match val.validate() {
            Ok(()) => Ok(Validated(val)),
            Err(err) => Err(err),
        }
    }

    fn inner(&self) -> &T {
        &self.0
    }
}

fn main() {
    let email = Email(String::from("test@test.com"));
    let port = Port(8080);

    let valid_email: Result<Validated<Email>, String> = Validated::new(email);
    let valid_port: Result<Validated<Port>, String> = Validated::new(port);

    match &valid_email {
        Ok(v) => println!("valid email: {}", v.inner().0),
        Err(err) => println!("{err}"),
    }

    match &valid_port {
        Ok(v) => println!("valid port: {}", v.inner().0),
        Err(err) => println!("{err}"),
    }

    // --- Proof that Validated<Email> and Validated<Port> are separate types ---
    // Uncommenting either line below fails to compile:
    //
    // let mismatched: Validated<Email> = valid_port.unwrap();
    // error[E0308]: mismatched types
    //   expected struct `Validated<Email>`, found struct `Validated<Port>`
    //
    // fn takes_email(_v: Validated<Email>) {}
    // takes_email(valid_port.unwrap());
    // error[E0308]: mismatched types
    //   expected struct `Validated<Email>`, found struct `Validated<Port>`
    //
    // Even though both are `Validated<T>` for some T, the compiler treats
    // Validated<Email> and Validated<Port> as completely distinct, unrelated
    // types — there is no implicit conversion between them, no shared
    // representation used interchangeably, and no way to pass one where the
    // other is expected. This is exactly the guarantee the newtype wrapper
    // is meant to provide: a `Validated<Port>` can never be mistaken for a
    // `Validated<Email>` at compile time, even though under the hood they're
    // both just "a T that passed validate()".
}
```

### What changed from your version

- `new` now returns `Result<Self, String>` and wraps the value as `Validated(val)` on success, instead of returning a hardcoded success message.
- Added `NonEmptyString` with its own `Validate` impl.
- Removed the redundant `is_empty()` check in `Email::validate` (already covered by `contains('@')`) and the redundant upper-bound check in `Port::validate` (already covered by `u16`'s range).
- Added the commented-out compile-error demonstration showing `Validated<Email>` and `Validated<Port>` are distinct, non-interchangeable types.
