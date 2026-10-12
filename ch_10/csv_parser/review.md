# Code review: `src/main.rs`

Reviewed at commit `a86fb42` ("CLP s10: query").

**Status:** all 45 tests pass. Clippy reports one lint, plus two dead-code
warnings that come from `main` not using the parser.

**Summary:** the parser itself (`new`, `next`, `headers`, `record`) is
correct for everything the tests cover, and it really is zero-copy. The
main problem is `query`: it doesn't use the parser at all. It has its own
second, simpler parser, and that parser breaks several spec rules,
including three inputs that **panic**. There is also one spec rule that
`next` gets wrong, and one that `new` gets wrong. The test suite doesn't
cover any of these cases, which is why everything is green. That gap is
mine (I wrote the tests), and section 5 lists the tests that would have
caught each bug.

Every bug below was confirmed by running your code on the input shown.

Severity: **High** = panic or wrong data. **Medium** = spec violation on
less common input. **Low** = style, readability, small efficiency.

---

## 1. Correctness

### 1.1 High: `query` panics on short rows and blank lines (R2, R6, R15)

| Input | Call | Result |
|---|---|---|
| `"a,b\n\n1,2\n"` | `query(_, "b", "2")` | **panic**: index out of bounds |
| `"a,b\n1\n1,2\n"` | `query(_, "a", "1")` | **panic** |
| `"a,b\n1\n"` | `query(_, "b", "x")` | **panic** |

**Why it happens:** `r[col_num]` (line 172) and `r[i]` (line 177) index
the row directly. That's fine only if every row has at least as many
fields as the header. A blank line splits into `[""]`, which has one
field. A short row has fewer fields than the header. Indexing a `Vec`
past its length panics.

- The first and third inputs panic on `r[col_num]` (line 172): the row
  has no field at the column's index.
- The second panics on `r[i]` (line 177). Column `a` exists in the short
  row and matches, but building the map then reads field 1, which isn't
  there.

**R2** says the parser never panics on any input. **R15** says ragged
rows are allowed.

**Direction:** never index a row with `[]` when the index might be out of
range. `Vec::get` returns an `Option` instead of panicking. For building
the map, `record` already handles short rows correctly (R20), so reuse it
(see 1.4).

### 1.2 High: `query` returns nothing for CRLF input (R3)

`query("a,b\r\n1,2\r\n", "b", "2")` returns `[]`. It should return one
match.

**Why:** `csv.split('\n')` leaves the `\r` at the end of each line. So
the last header is `"b\r"`, not `"b"`, and `position` never finds column
`"b"`. Even when the column is found, the last field of every row has a
`\r` stuck to it, so values like `"2\r"` never equal `"2"`. `next`
already strips the `\r` correctly; `query` doesn't benefit from that
because it doesn't use `next`.

### 1.3 High: `query` treats blank lines as rows (R6)

`query("a\n1\n\n", "a", "")` returns `[{"a": ""}]`. It should return
`[]`, because the only data row is `1`.

**Why:** lines 158-160 remove only **one** trailing empty string. The
input above ends with `\n\n`, so `split('\n')` gives
`["a", "1", "", ""]`. After one `pop`, the `""` that's left is treated
as a row with one empty field, and it matches `value == ""`. A blank line
in the middle of the input has the same effect (and panics instead when
the header has more than one column, see 1.1).

### 1.4 High (root cause of 1.1-1.3): `query` re-implements parsing

`query` splits the input on its own instead of using `CsvParser`. So
every rule that `new` and `next` handle (CRLF, blank lines, ragged rows)
has to be re-implemented, and the copy handles none of them. You noted
this yourself in the comment on line 142.

This is the most important fix in this review. Build `query` on top of the
parser:

- `CsvParser::new(csv)` for the headers.
- Look up the column's index in `headers()`. Unknown column: return early,
  as you already do with `let ... else`.
- Iterate the parser for rows. For each row, check the field at that
  index without panicking (1.1).
- For matching rows, call `record` to build the map.

When this is done, 1.1, 1.2, 1.3 and 1.7 are fixed with no extra code,
and `query` becomes about half its current length. Iterator adapters
(`filter`, `map`, `collect`) make it shorter still, but a `for` loop is
fine.

### 1.5 Medium: a lone `\r` at the end of the input is dropped (R4)

`rows("a\n1\r")` returns `[["1"]]`. It should return `[["1\r"]]`.

**Why:** in `next`, when there is no `\n` left, `pos` becomes the length
of the rest of the input (line 101). The `\r` check on lines 106-110 then
looks at the last byte of the input. It finds the `\r` and removes it,
even though no `\n` follows it. Per R3 and R4, only `\r\n` is a line
ending; a `\r` without `\n` is data.

The check should only strip `\r` when the line really ended at a `\n`.
The existing test `s05_lone_carriage_return_is_ordinary_data` passes
because there the `\r` is in the middle of a line, not at the end of the
input.

### 1.6 Medium: a blank line before the header becomes the header (R10)

| Input | Result | Expected |
|---|---|---|
| `"\na,b\n1,2\n"`, `headers()` | `[""]` | `["a", "b"]` |
| `"\na,b\n1,2\n"`, rows | `[["a","b"], ["1","2"]]` | `[["1","2"]]` |
| `"\n"`, `headers()` | `[""]` | `[]` |

**Why:** `new` takes the first line as the header without checking
whether it's blank. The blank-skipping loop (lines 43-57) runs only
*after* the header has been read. R10 says the header is the first
**non-blank** line.

`next` already knows how to skip blank lines. If `new` and `next` shared
one "read the next non-blank line" helper (see 3.1), this would be fixed
automatically.

### 1.7 Medium: `query` reads the header from a blank first line

`query("\na,b\n1,2\n", "a", "1")` returns `[]` instead of one match. This
has the same cause as 1.6, but it lives in `query`'s own copy of the
parsing (`rows[0]`, line 162). Fixing 1.4 removes it.

---

## 2. Efficiency

The parser's core is efficient. `find('\n')` on a `str` uses a fast byte
search, every line is scanned only a small, constant number of times, and
there are no `String` allocations. The points below are small, except the
first one.

### 2.1 `query` reads the whole input up front

Line 155 collects **every** line into a `Vec` before checking any of
them. That is one pass over the input plus a heap allocation sized to the
number of lines. Then it splits every row into another `Vec`, even rows
that will not match.

Using the parser (1.4) makes this lazy: one row at a time, constant extra
memory apart from the results. If you want to go further, you don't need
to split a whole row to check one column. You only need the field at
`col_num`, and an iterator can stop there. That's an optional
micro-optimization, and it doesn't fit the current API (the iterator
already returns a full `Vec` per row), so it's fine to skip.

### 2.2 `new` searches for the first `\n` twice

Lines 16-17 and line 39 both call `input.find('\n')` on the same input.
Find it once and derive both values (the end of the header text and the
start of the next line) from that one result. This is the same "two
positions, two jobs" idea from before: one search, two derived values.

### 2.3 `record` can reserve the map's size

`HashMap::new()` starts empty and grows (re-allocates) as you insert.
You know the final size in advance, so `HashMap::with_capacity` avoids
the regrowth. Even simpler: build it with `zip` and `collect` (see 3.3).
`collect` uses the iterator's length hint to reserve space for you.

---

## 3. Design and readability

### 3.1 Duplicate line-reading logic in `new` and `next`

These pieces exist twice, almost word for word:

- Finding the end of a line.
- Handling `\r\n`.
- The blank-line-skipping loop: lines 43-57 and 120-130.

Duplicate code drifts apart. Bug 1.6 is exactly that: `new` and `next`
handle blank lines differently.

Extract one private helper on `CsvParser` that returns the next non-blank
line (as a `&'a str`) and advances `pos`. Then:

- `new` calls it once for the header.
- `next` calls it and splits the result on `,`.

This fixes 1.5 and 1.6 in one place, and `next` shrinks to a few lines.

### 3.2 Keep `pos <= input.len()` always true

When there is no `\n`, line 39 sets `pos` to `input.len() + 1`, one byte
**past** the end. It works today only because every later check is
`pos < len` or `pos >= len`. But any future `&self.input[self.pos..]`
would panic. Make "`pos` never exceeds `input.len()`" a rule for the
struct, and keep it true everywhere.

### 3.3 Simpler expressions

- **Line 74-78:** the `if`/`else` that computes `smaller_len` is
  `usize::min`.
- **Lines 80-82:** clippy flags `needless_range_loop`. Indexing with `i`
  over `0..len` is a C-style loop. The idiomatic Rust version pairs the
  two sequences with `zip`, which also stops at the shorter one. That
  means you don't need `smaller_len` at all. In this case, the whole body
  of `record` can be one expression.
- **Lines 105-113:** `let mut row_end;` followed by an `if` that assigns it
  can be `let row_end = if ... { ... } else { ... };`, because `if` is an
  expression in Rust. It doesn't need to be `mut` either.
- **Lines 96-117:** a lot of index arithmetic (relative to the slice vs.
  relative to the input) exists only to strip a trailing `\r`. Once you
  have the line as a `&str`, `str::strip_suffix` removes a suffix in one
  call, with no index math. You already use `strip_prefix` the same way in
  the blank-line loop.

### 3.4 Comments

- **Line 11:** `/n` should be `\n`.
- **Lines 33-34:** commented-out code plus a "prolly should" note. Delete
  them; git keeps the history.
- **Line 61:** `pos` holds the start of the first **data** row, not "the
  end of headers". A wrong comment is worse than none.
- **Line 97:** says "position of the last element in the row", but the
  value is the position of the `\n`, relative to the slice.
- **Lines 141-142:** "LATRER" is a typo. When you fix 1.4, delete the note.
- **Lines 150-154 and 173-174:** questions to yourself. They're answered
  in section 4 below; replace them with a short explanation or remove
  them.
- **Lines 198-200:** commented-out copy of the test helper. Remove it.

### 3.5 `main`

`main` is a scratch demo. It's fine to keep, but it's why `cargo clippy`
reports the parser as "never constructed": `main` calls only `query`, and
`query` doesn't use the parser. When `query` uses `CsvParser` (1.4), those
two warnings go away too.

---

## 4. Answers to the questions in your comments

### Line 150: "this is stored on the stack right? ... how can a vec be stored on the stack?"

The answer is half right. A `Vec` has two parts:

- **On the stack:** a small header of three numbers: a pointer, the
  length, and the capacity (24 bytes on 64-bit).
- **On the heap:** the buffer that holds the elements.

Here the elements are `&str` values, each one a pointer plus a length (16
bytes). So `rows` **does** allocate on the heap: an array of
16-byte references, one per line. What it does *not* allocate is any copy
of the text. The references point into `csv`.

That's exactly the line the spec draws (R17): allocating `Vec`s of
references is fine, and copying string data is not. "Zero-copy" means
zero copies of the text, not zero heap allocations.

### Line 173: "how does this live for as long as res? is this not if-block scoped?"

The **variable** `row_matches` is scoped to the block. The **value** is
not. `res.push(row_matches)` *moves* the `HashMap` into `res`: ownership
transfers (TRPL ch4), and the `HashMap` now belongs to `res`. When the
block ends, the variable name goes away, but it no longer owns anything,
so nothing is dropped. The map lives as long as `res` does.

The `&'a str` keys and values inside the map are fine for a different
reason: they borrow from `csv`, not from anything local to the block.
`csv` outlives the function, and the `'a` in the return type tells the
compiler so.

---

## 5. Missing tests

Each bug above went unnoticed because no test covers it. Add these
**first**, watch them fail, then fix the code. That's the same red-green
loop as before. Suggested names and the rule each one checks:

| Test | Input | Expected | Rule |
|---|---|---|---|
| `s05_lone_carriage_return_at_end_of_input` | `rows("a\n1\r")` | `[["1\r"]]` | R4 |
| `s06_blank_lines_before_header_are_skipped` | `"\n\na,b\n1,2\n"` | headers `["a","b"]`, rows `[["1","2"]]` | R6, R10 |
| `s06_only_blank_lines` | `"\n\n"` | headers `[]`, rows `[]` | R6, R12 |
| `s10_query_crlf` | `query("a,b\r\n1,2\r\n", "b", "2")` | one match, `{"a":"1","b":"2"}` | R3 |
| `s10_query_skips_blank_lines` | `query("a,b\n\n1,2\n\n", "b", "2")` | one match, no panic | R2, R6 |
| `s10_query_blank_line_does_not_match_empty_value` | `query("a\n1\n\n", "a", "")` | `[]` | R6 |
| `s10_query_short_row_does_not_panic` | `query("a,b\n1\n1,2\n", "b", "2")` | one match: the `1,2` row | R2, R15 |
| `s10_query_column_missing_in_short_row` | `query("a,b\n1\n", "b", "x")` | `[]` | R2 |

A broader, optional check: a property test (`proptest` crate) that feeds
random strings into `rows` and `query` and asserts only "doesn't panic".
It would have found 1.1 on its own.

---

## 6. What's done well

- **Zero-copy is real.** Every returned `&str` is a slice of the input.
  The lifetimes are correct, and the commented-out lifetime test fails to
  compile as intended.
- **You spotted the `pos - 1` underflow** and guarded it (lines 103-113).
  That's the kind of edge case that crashes production parsers.
- **`strip_prefix`** is the right tool for the blank-line loop: no manual
  byte comparisons.
- **`let ... else`** in `query` for the unknown-column case, and
  `iter().position(...)` to find the column. Both idiomatic.
- **`next` keeps returning `None` after the end** (R14) without needing
  an extra flag.
- **Commit history:** one commit per stage, with the stage name in the
  message. That's a good habit.

---

## Suggested order of work

1. Add the tests from section 5 and confirm they fail.
2. Extract the line-reading helper (3.1) and keep `pos <= len` (3.2). This
   fixes 1.5 and 1.6.
3. Rewrite `query` on top of the parser (1.4). This fixes 1.1, 1.2, 1.3,
   1.7 and 2.1.
4. Clean up `record` (3.3) and the comments (3.4).
5. Run `cargo test` and `cargo clippy`. Both should be clean.
