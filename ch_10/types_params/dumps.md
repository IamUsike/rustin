- generic means _the caller picks the type, and the code has to work for every possible choice_

### 1. Generics are params, but for types

```rust
fn double(x: i32) -> i32; //x is a value param
fn pick<T>(x: T) -> T; //t is a type param
```

- whoever calls pick supplies T, and the fn body has to work for whatever they supply.
- Traits work the same way, **except, the "caller" is the impl block.**

### 2. Three different places <...> shows up

```rust
trait Transform<In, Out> {          // (1) DECLARATION
    fn apply(&self, x: In) -> Out;
}

struct Doubler;

impl<T> Transform<T, i64> for Doubler { ... }
//  ^^^ (2) names I'm declaring     ^^^^^^^^^^^^^^ (3) filling the trait's slots
```

1. `trait Transform<In, Out>` declares two empty slots. Nothing is decided yet.

2. `impl<T>` declares names that are open inside this one impl. Read it as "for every type `T`..." Anything not listed here is not open.

3. `Transform<T, i64>` fills each slot. A slot can hold either a name from (2), which keeps it open, or a concrete type, which pins it.

So that impl line reads: "for every type `T`, `Doubler` implements `Transform<T, i64>`." The `T` is open, and the `i64` is pinned.

---

Close, but let me sharpen it slightly — the rule isn't "pin `Output`, leave `Input` open." It's "pin whichever type your body has to _construct_, and you can leave open whichever type your body only _receives and uses through bounds_."

Usually the output is the thing you construct, so in practice, yes, `Output` tends to be the one you pin. But it's not automatic — it depends on what the body actually does.

**Case 1: body builds a value from scratch → must pin**

```rust
impl<T> Transform<T, Vec<String>> for Doubler {
    fn apply(&self, x: T) -> Vec<String> {
        vec![String::from("hi")]   // hardcoded — only works as Vec<String>
    }
}
```

`Out` had to be pinned because the body decides exactly what comes back, with no way to produce an arbitrary type.

**Case 2: body can construct the generic type, if the bounds allow it**

```rust
impl<T: Default> Transform<i32, T> for Doubler {
    fn apply(&self, x: i32) -> T {
        T::default()   // works for ANY T that implements Default
    }
}
```

Here `Out` stays open, because the bound (`Default`) gives the body a recipe for building a `T`, whatever `T` turns out to be. This is rarer, but it's the reason "always pin the output" isn't a hard law — it's a consequence of not having a bound that lets you manufacture the value.

**Case 3: `Input` needs pinning too, if you do something that requires knowing the concrete type**

```rust
impl Transform<MySpecialInput, i64> for Doubler {
    fn apply(&self, x: MySpecialInput) -> i64 {
        x.some_field_only_this_type_has   // needs the exact type, not a bound
    }
}
```

**The actual test:** for each slot, ask "does my body know how to produce/handle every possible type here, using only trait bounds?" If yes, leave it open. If no — if the body only makes sense for one specific type — pin it.

For your `CsvLineParser`, apply that test to `Output`. Your body always returns a `Vec<String>`, and there's no bound that would let it return an arbitrary `Output`. So per the test, what should happen to that slot?
