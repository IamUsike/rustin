# Code review: associated types vs generic params

Target: `src/main.rs`. Task per README: implement `Parser` two ways (generic
params, associated types), build `CsvLineParser` + `JsonLineParser` for both,
write `run_parser`, and comment on when each version is appropriate.

## Verdict

Active code (M2, associated-type version) compiles clean, `cargo clippy`
finds nothing, logic is correct. Confirmed by running it. But the exercise
asks for **both** versions side by side plus a written comparison — only one
is live, the other is commented-out dead code, and the comparison comment is
missing entirely. That's the actual point of the exercise, so it's not done.

## Got right

- **Associated-type `Parser` (M2) is correct.** `type Input`/`type Output`
  pinned per impl, `parse` signatures match, both parsers produce right
  output (`Vec<String>` for CSV, `Vec<(String,String)>` for JSON-lines).
- **`run_parser` bound is the textbook-correct form**:
  `impl Parser<Input = Input, Output = Output>` — this is exactly the
  ergonomic win the README wants you to notice.
- **`filter_map` + `split_once` for the JSON parser** is idiomatic, no manual
  indexing, no `unwrap`.
- Comments on `AsRef<str>` in the dead V1 code show you understood *why*
  it's there (cheap ref-to-ref conversion, caller picks `Input`).
- Verified V1 (generic-param version) independently: it compiles and runs
  fine standalone. Your implementation logic there is also correct.

## Got wrong / missing

### 1. Only one version is live — can't actually compare them

V1 sits commented out. You can't run both, so the exercise's whole point
("see why Version B is easier to use as a bound") never gets demonstrated in
your own code, only asserted in the README's prose.

Fix: give the two traits different names so both can exist and be
implemented by the same structs at once.

```rust
// Version A: generic params — one impl PER (Input, Output) pair.
trait ParserGeneric<Input, Output> {
    fn parse(&self, input: Input) -> Option<Output>;
}

// Version B: associated types — one impl per type, period.
trait ParserAssoc {
    type Input;
    type Output;
    fn parse(&self, input: Self::Input) -> Option<Self::Output>;
}

struct CsvLineParser;

impl ParserGeneric<String, Vec<String>> for CsvLineParser {
    fn parse(&self, input: String) -> Option<Vec<String>> {
        Some(input.split(',').map(str::to_owned).collect())
    }
}

impl ParserAssoc for CsvLineParser {
    type Input = String;
    type Output = Vec<String>;
    fn parse(&self, input: String) -> Option<Vec<String>> {
        Some(input.split(',').map(str::to_owned).collect())
    }
}
```

### 2. Missing the required comment on when each is appropriate

README asks for this explicitly; it's not in the file. Add it near the trait
defs, e.g.:

```rust
// Generic params (ParserGeneric<Input, Output>): use when ONE type needs
// MULTIPLE impls of the same trait — e.g. From<T> impl for many T,
// Add<Rhs> for several Rhs types. Cost: every caller/bound must restate
// both type params (`impl ParserGeneric<String, Vec<String>>`), and if a
// type has 2+ impls, Output can't be inferred from Input alone.
//
// Associated types (ParserAssoc): use when a type has exactly ONE sensible
// Input/Output pair — Iterator::Item, Add::Output. Cost: locks the type to
// one impl. Benefit: bounds shrink to `impl ParserAssoc`, no type params to
// restate, and `Self::Output` reads cleaner in the impl itself.
```

### 3. Same struct names reused across both attempts

Not a bug (a struct can implement two differently-named traits), but if you
follow fix #1, keep `CsvLineParser`/`JsonLineParser` shared across both
`impl` blocks — that's actually the right move, don't duplicate structs.

### 4. Minor: double `.replace()` pass

```rust
// current — two passes over the string
let input = input.replace("{", "").replace("}", "");
```
```rust
// one pass — replace() takes a Pattern, arrays of char implement it
let input = input.replace(['{', '}'], "");
```

### 5. No runnable check

Both parsers have real branching logic (empty-input guard, split/filter_map)
and nothing asserts their output. Add one `#[test]` block, no framework
needed:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_splits_on_comma() {
        let p = CsvLineParser;
        assert_eq!(p.parse("a,b".into()), Some(vec!["a".into(), "b".into()]));
    }

    #[test]
    fn json_line_pairs_kv() {
        let p = JsonLineParser;
        assert_eq!(
            p.parse("{a:b,c:d}".into()),
            Some(vec![("a".into(), "b".into()), ("c".into(), "d".into())])
        );
    }
}
```

### 6. Cosmetic

- `"Provid a valid json"` — typo, existing.
- `//////// M2` section header — fine, but once V1 is deleted (not just
  commented) this marker has no reason to exist.

## Skipped in this review

Didn't touch error handling (`Option` vs `Result`) — README doesn't ask for
it, `Option` is fine for a toy parser. Add `Result<_, ParseError>` only if
you need to say *why* parsing failed.
