## Code Review — Vec2 Operator Overloading

### ✅ What's solid

- **All six trait impls are correct**: `Add`, `Sub`, `Neg`, `Mul<f64>`, `Div<f64>`, `PartialEq` all have the right signatures and correct arithmetic.
- **`From<(f64,f64)>`** is correct, and you correctly did _not_ try to hand-write `Into` — the blanket impl `impl<T, U> Into<U> for T where U: From<T>` gives you that for free. Good instinct not to duplicate it.
- **`dot(&self, &Self) -> f64`** taking references instead of consuming `self` is a nice, deliberate choice — dot product doesn't need to own its operands, unlike `add`/`sub`/`neg` which naturally consume and return a new `Vec2`. Mixing by-value and by-reference APIs where it makes sense shows you're thinking about ownership semantics, not just copy-pasting the trait signature.
- **`Display`** formatting is exactly the "(x, y)" spec.
- Test result is correct: `(3,4)*2 + (1,0) = (7,8)`, `dot (1,0) = 7`. Your `println!` confirms `7`.

### 🔧 Corrections / suggestions

1. **The prompt asked for a `Test:` — you have a `println!`, not an `assert_eq!`.** Since `dot()` returns `f64` (not `Vec2`), you don't even need `Debug` on `Vec2` to assert on it:

```rust
   assert_eq!(
       (Vec2::from((3.0, 4.0)) * 2.0 + Vec2::from((1.0, 0.0))).dot(&Vec2::from((1.0, 0.0))),
       7.0
   );
```

This actually verifies correctness instead of requiring a human to eyeball stdout.

2. **No `#[derive(Debug, Copy, Clone)]` on `Vec2`.** Not strictly required, but:
   - `Debug` costs nothing and is the idiomatic default on any struct you'll ever print/assert/debug.
   - `Copy` is reasonable here (two `f64`s, cheap, no ownership-sensitive resources) and would let you reuse a `Vec2` after passing it into `normalize()` without a move error. Right now, `normalize(v)` consumes `v` permanently — if you tried to use `v` again afterward, it wouldn't compile.

3. **Missing the reverse-order scalar multiply.** You implemented `Vec2 * f64` (`impl Mul<f64> for Vec2`), but `2.0 * v` won't compile — `Mul` isn't commutative for you automatically. If you want both directions (common in vector-math libraries), you'd need:

```rust
   impl Mul<Vec2> for f64 {
       type Output = Vec2;
       fn mul(self, v: Vec2) -> Vec2 { v * self }
   }
```

4. **`normalize` has a latent divide-by-zero bug.** `normalize(Vec2 { x: 0.0, y: 0.0 })` divides by `0.0`, giving `NaN` components silently. Not wrong per the spec, but worth at least a comment or a guard:

```rust
   fn normalize(v: Vec2) -> Vec2 {
       let mag = (v.x * v.x + v.y * v.y).sqrt();
       v / mag  // NaN if mag == 0.0 — caller's responsibility, or add a check
   }
```

5. **Minor style: `let mut d = ...; d = d.sqrt();`** works but reassigning a `mut` just to immediately transform it once is more naturally written as a single expression:

```rust
   let mag = (v.x * v.x + v.y * v.y).sqrt();
```

Also — you called it `d` for "distance"/magnitude but then used `v.div(d)` instead of the more idiomatic `v / d` (you already have `Div` implemented via operator syntax, may as well use it since that's the whole point of the exercise).

6. **Float `PartialEq` note (not a bug, just worth knowing):** `self.x == other.x` is exact bitwise float equality. Fine for this exercise, but in real code accumulated floating-point error means two "equal" vectors from different computation paths often won't compare equal — usually handled with an epsilon-based `approx_eq` helper instead of `PartialEq` for floats.

### Summary

Correct and idiomatic in the core trait mechanics — the `dot` reference-vs-value split especially shows good judgment. Main gaps: the test isn't actually asserting anything, and a couple of small robustness/derive omissions (`Copy`, `Debug`, reverse-order
`Mul`, div-by-zero) that a stricter reviewer would flag.
