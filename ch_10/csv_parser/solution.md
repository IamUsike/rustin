# Reference solution

This is the corrected and simplified version of `src/main.rs`, following
`review.md`. It's a separate file so your own code stays untouched: read
it, compare, and port what you want by hand.

**Verified:** I ran this code against your current `src/tests.rs` plus the
8 new tests from review.md section 5 (included at the end of this file).

- 53 of 53 tests pass.
- Clippy reports only the dead-code warnings caused by an empty `main`.
- The lifetime check (`s08_rows_cannot_outlive_input`) still fails to
  compile, as intended.

Where it differs from your version:

| Review item | What changed |
|---|---|
| 3.1, 1.5, 1.6 | One `next_line` helper reads lines for both `new` and `next` |
| 3.2 | `pos` never goes past `input.len()` |
| 3.3, 2.3 | `record` is a single `zip` + `collect` |
| 1.4, 1.1-1.3, 1.7, 2.1 | `query` is built on the parser and uses `get` instead of `[]` |
| 2.2 | The input is scanned once. `new` no longer searches for `\n` twice |

---

## Code

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct CsvParser<'a> {
    input: &'a str,
    /// Byte offset where the next unread line starts. Always <= input.len().
    pos: usize,
    headers: Vec<&'a str>,
}

impl<'a> CsvParser<'a> {
    fn new(input: &'a str) -> Self {
        let mut parser = CsvParser {
            input,
            pos: 0,
            headers: Vec::new(),
        };
        // The header is the first non-blank line (R10). No line at all means
        // no headers (R12).
        if let Some(line) = parser.next_line() {
            parser.headers = line.split(',').collect();
        }
        parser
    }

    fn headers(&self) -> &[&'a str] {
        &self.headers
    }

    fn record(&self, row: Vec<&'a str>) -> HashMap<&'a str, &'a str> {
        // zip stops at the shorter side, which is exactly R20.
        self.headers.iter().copied().zip(row).collect()
    }

    /// Returns the next non-blank line without its line ending, and moves
    /// `pos` to the start of the line after it.
    fn next_line(&mut self) -> Option<&'a str> {
        let input = self.input; // copy the &'a str so returned slices borrow from the input, not from self
        while self.pos < input.len() {
            let rest = &input[self.pos..];
            let (line, consumed) = match rest.find('\n') {
                // Only strip '\r' when the line really ended in "\r\n" (R3, R4).
                Some(i) => (rest[..i].strip_suffix('\r').unwrap_or(&rest[..i]), i + 1),
                None => (rest, rest.len()),
            };
            self.pos += consumed;
            if !line.is_empty() {
                return Some(line);
            }
            // Blank line: keep looking (R6).
        }
        None
    }
}

impl<'a> Iterator for CsvParser<'a> {
    type Item = Vec<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_line().map(|line| line.split(',').collect())
    }
}

fn query<'a>(csv: &'a str, column: &str, value: &str) -> Vec<HashMap<&'a str, &'a str>> {
    let mut parser = CsvParser::new(csv);
    let Some(col) = parser.headers().iter().position(|h| *h == column) else {
        return Vec::new();
    };

    let mut matches = Vec::new();
    while let Some(row) = parser.next() {
        // `get` returns None for short rows instead of panicking (R2, R15).
        if row.get(col) == Some(&value) {
            matches.push(parser.record(row));
        }
    }
    matches
}
```

---

## Walkthrough

### `next_line`: the one place that understands lines

All the line rules live here and nowhere else:

- The line ends at the next `\n`, or at the end of the input.
- A `\r` directly before that `\n` is removed.
- Blank lines are skipped.

**Why `strip_suffix` sits only in the `Some(i)` arm:** that's the fix for
review 1.5. In the `Some(i)` arm we know a `\n` follows the line, so a
`\r` right before it is part of a `\r\n` ending. In the `None` arm there
is no `\n`, so a trailing `\r` is data (R4) and stays.

**`strip_suffix(...).unwrap_or(...)`:** `strip_suffix` returns
`Some(shorter)` if the suffix was there, and `None` otherwise.
`unwrap_or` falls back to the original slice. This replaces all the index
arithmetic around `row_end` in your version.

**`consumed`** is how far `pos` moves: the line plus its `\n`, or the rest
of the input if there's no `\n`. `pos` can therefore reach `input.len()`
but never go past it (review 3.2). And once `pos == input.len()`, the
`while` condition is false, so the method returns `None` forever (R14).

**Why `let input = self.input;` on the first line.** This is a lifetime
detail, and it's worth understanding:

- `self.input` has type `&'a str`. Shared references are `Copy`, so
  `input` is a copy of the same reference, with the same lifetime `'a`.
- Slices taken from `input` are `&'a str`, tied to the original text and
  independent of `self`. That's why the function can return
  `Option<&'a str>` while it also mutates `self.pos`.
- Writing `&self.input[self.pos..]` directly also compiles here: the
  compiler can see that the data behind a shared reference lives for
  `'a`. The copy just makes it obvious to the reader that the result
  borrows from the input, not from the parser.

### `new`: the header is "the first line `next_line` returns"

`new` builds the parser with `pos: 0`, then asks `next_line` for the first
non-blank line. Results:

- Blank lines before the header are skipped automatically (fixes review 1.6).
- Empty input, or input with only blank lines, gives no line. `headers`
  stays empty (R12).
- After `new`, `pos` already points at the first data row, so `next`
  never sees the header (R11).

### `next`: one line

`next_line` returns `Option<&str>`. `map` applies the split only when
there is a line, and passes `None` through unchanged. `collect` builds
the `Vec<&'a str>`.

### `record`: `zip` instead of an index loop

- `self.headers.iter()` yields `&&'a str`. `.copied()` turns it into
  `&'a str`. Copying a reference copies the pointer, not the text, so
  this is still zero-copy.
- `.zip(row)` pairs header 0 with field 0, header 1 with field 1, and so
  on, and **stops when either side runs out**. That's R20 with no
  `min` and no length check.
- `.collect()` builds the `HashMap`, because the return type says so. It
  also uses the iterator's size hint to reserve space up front
  (review 2.3).

### `query`: reuse the parser

1. `CsvParser::new(csv)` handles the header, blank lines, and CRLF.
2. `position` finds the column index. An unknown column returns early
   (R22). This part is the same as yours.
3. `while let Some(row) = parser.next()` walks the rows lazily: one row in
   memory at a time, no up-front `Vec` of every line (review 2.1).
4. `row.get(col)` returns `Option<&&str>`. It's `None` when the row is
   too short, so `== Some(&value)` is simply `false` instead of a panic
   (fixes review 1.1).
5. `parser.record(row)` builds the map, including the short-row rule.

**Why a `while let` loop and not `filter` / `map` / `collect`?** You might
expect to write something like "`parser`, filter by column, map through
`record`, collect". That doesn't compile:

- Iterating the parser needs a **mutable** borrow of `parser`, because
  `next` takes `&mut self`.
- The `map` closure calls `parser.record`, which needs a **shared**
  borrow of `parser`.
- Both borrows would be alive at the same time, and the borrow checker
  rejects that (TRPL ch4: one `&mut` or any number of `&`, never both).

The `while let` loop avoids the overlap. `parser.next()` borrows mutably
only for the duration of that call. By the time `record` runs, the
mutable borrow is over. An iterator chain *is* possible, by cloning the
headers into a local first, but the loop is simpler and doesn't copy
anything.

---

## Complexity

- **Time:** O(n) in the size of the input. Each byte is looked at a small,
  constant number of times: once by `find('\n')`, once by `split(',')`.
- **Memory:** no `String` is ever allocated. Per row, the parser allocates
  one `Vec` of references (the API requires that). `query` keeps only the
  matching rows, as `HashMap`s of references.

One optional micro-optimization, not applied: `query` splits the whole
row into a `Vec` even though it only needs one field to decide whether
the row matches. Doing better would require a second, non-allocating way
to read rows, which goes beyond this API. Not worth it here.

---

## The 8 new tests

These are the tests from review.md section 5. Paste them at the end of
`src/tests.rs`, below the s10 tests. Against your current code, they fail
(several by panicking). Against the code above, they pass.

```rust
// ---------- review: gaps found in code review ----------

#[test]
fn s05_lone_carriage_return_at_end_of_input() {
    assert_eq!(rows("a\n1\r"), vec![vec!["1\r"]]);
}

#[test]
fn s06_blank_lines_before_header_are_skipped() {
    let input = "\n\na,b\n1,2\n";
    assert_eq!(CsvParser::new(input).headers(), &["a", "b"]);
    assert_eq!(rows(input), vec![vec!["1", "2"]]);
}

#[test]
fn s06_only_blank_lines() {
    assert!(CsvParser::new("\n\n").headers().is_empty());
    assert_eq!(rows("\n\n"), Vec::<Vec<&str>>::new());
}

#[test]
fn s10_query_crlf() {
    let result = query("a,b\r\n1,2\r\n", "b", "2");
    assert_eq!(result, vec![HashMap::from([("a", "1"), ("b", "2")])]);
}

#[test]
fn s10_query_skips_blank_lines() {
    let result = query("a,b\n\n1,2\n\n", "b", "2");
    assert_eq!(result, vec![HashMap::from([("a", "1"), ("b", "2")])]);
}

#[test]
fn s10_query_blank_line_does_not_match_empty_value() {
    assert!(query("a\n1\n\n", "a", "").is_empty());
}

#[test]
fn s10_query_short_row_does_not_panic() {
    let result = query("a,b\n1\n1,2\n", "b", "2");
    assert_eq!(result, vec![HashMap::from([("a", "1"), ("b", "2")])]);
}

#[test]
fn s10_query_column_missing_in_short_row() {
    assert!(query("a,b\n1\n", "b", "x").is_empty());
}
```

---

## Small leftover in `src/tests.rs`

Line 56 has a `dbg!(&p);` inside `s01_empty_input`, left over from
debugging. It's why the struct needs `#[derive(Debug)]`. Remove it when
you're done. The derive is harmless to keep either way.
