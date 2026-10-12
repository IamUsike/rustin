# CsvParser spec

This is the contract that `src/tests.rs` enforces. If a test and this file
disagree, one of them has a bug; fix it before you write more code.

## 1. API (signatures only)

```
struct CsvParser<'a>
CsvParser::new(input: &'a str) -> CsvParser<'a>
CsvParser::headers(&self) -> &[&'a str]
CsvParser::record(&self, row: Vec<&'a str>) -> HashMap<&'a str, &'a str>
impl Iterator for CsvParser<'a> { type Item = Vec<&'a str>; }
fn query<'a>(csv: &'a str, column: &str, value: &str) -> Vec<HashMap<&'a str, &'a str>>
```

## 2. Rules the parser follows

### Input

- R1. Input is a `&str`. It is valid UTF-8, so no encoding checks are needed.
- R2. The parser never panics, for any input, including `""`.

### Lines

- R3. A line ends at `\n` or `\r\n`. Neither the `\n` nor the `\r` belongs to any field.
- R4. A `\r` that is not directly followed by `\n` is ordinary data.
- R5. The last line does not need a line ending.
- R6. Blank lines (zero characters before the line ending) are skipped everywhere:
  before the header, between rows, and at the end.

### Fields

- R7. The only delimiter is `,`.
- R8. A line with N commas has exactly N + 1 fields. So `1,` is `["1", ""]`
  and `,,` is `["", "", ""]`.
- R9. Fields are returned exactly as written. No trimming, no unquoting.

### Header

- R10. The first non-blank line is the header. `new` reads it immediately, so
  `headers()` works before any iteration.
- R11. The iterator never yields the header row.
- R12. Empty input gives empty headers and no rows.

### Iterator

- R13. Rows are produced lazily: one call to `next` parses one row.
- R14. After `next` returns `None`, every later call also returns `None`.
- R15. Rows can have a different field count than the header. They are
  yielded as they are, with no error.

### Zero-copy

- R16. Every `&str` returned (headers, fields, `record` keys and values,
  `query` keys and values) is a sub-slice of the original input. Same
  memory, not a copy with equal text. This also applies to empty fields:
  return an empty slice of the input, not the literal `""`.
- R17. No `String` is created for field data. Allocating the `Vec`s and
  `HashMap`s is fine: they hold the slices, not copies of the text.
- R18. A row cannot outlive the input. The compiler must reject that.

### record

- R19. `record` pairs headers with fields by position.
- R20. If the row is shorter than the header, the missing columns are left
  out. If the row is longer, the extra fields are left out.

### query

- R21. `query` returns, in input order, every row whose value in `column`
  equals `value` exactly (case-sensitive, whole string).
- R22. An unknown column, no matches, or empty input gives an empty `Vec`.
- R23. The results borrow only from `csv`, never from `column` or `value`.

## 3. Out of scope (do NOT implement)

| Feature                      | What happens instead                                                                                                                                   |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Quoted fields (`"a,b"`)      | Quotes are ordinary characters, and a comma inside quotes still splits (R9). With `Vec<&'a str>` it is impossible to unescape `""` without allocating. |
| Other delimiters (`;`, tab)  | Only `,` (R7).                                                                                                                                         |
| Whitespace trimming          | Fields are kept as written (R9).                                                                                                                       |
| Errors / `Result`            | There are none: every input parses (R2, R15).                                                                                                          |
| Duplicate header names       | Not tested. With `HashMap`, the last one wins.                                                                                                         |
| Reading from files or `Read` | Input is always a `&str` already in memory.                                                                                                            |
| Comments (`#`), BOM handling | Treated as ordinary data.                                                                                                                              |

## 4. Workflow

### Step 0: get to compiling

The test module only compiles when every function in section 1 exists.
Write all the signatures first, each with a `todo!()` body. Then
`cargo test` compiles, and every test fails because of the `todo!()`
panic, not because of a compile error. This is the "red" starting point.

### Step 1: go through the stages in order

Run one stage at a time. `cargo test` treats its argument as a name filter:

```
cargo test s01        # run only the stage 1 tests
cargo test s05_crlf   # run a smaller group
cargo test            # run everything (do this before every commit)
```

| Stage | Covers                          | Rules      |
| ----- | ------------------------------- | ---------- |
| s01   | `new`, `headers`, empty input   | R10–R12    |
| s02   | one data row                    | R3, R5, R8 |
| s03   | many rows, `None` after the end | R13, R14   |
| s04   | empty fields                    | R8         |
| s05   | CRLF                            | R3, R4     |
| s06   | blank lines                     | R6         |
| s07   | out-of-scope behavior stays out | R9, R15    |
| s08   | zero-copy                       | R16–R18    |
| s09   | `record`                        | R19, R20   |
| s10   | `query`                         | R21–R23    |

The loop for each stage:

1. Run `cargo test sNN` and read the failures (see below).
2. Write the smallest change that turns them green.
3. Run the full `cargo test`. Earlier stages must still pass.
4. Refactor if the code got ugly. Run `cargo test` again.
5. Commit, for example `git commit -m "s04: empty fields"`.

Some stages may already be green when you reach them, because of earlier
work. That is fine. Read those tests anyway, so you know what they protect.

### How to read a failure

`assert_eq!` prints both sides:

```
assertion `left == right` failed
  left: [["1", "2\r"]]
 right: [["1", "2"]]
```

- `left` is what your parser returned. `right` is what the spec expects.
- Strings are printed with `{:?}`, so hidden characters show up: `"2\r"`
  means a carriage return leaked into the field (R3).
- The test name tells you the rule. For example,
  `s05_crlf_headers_have_no_carriage_return` points to R3.

A panic means something different:

- `not yet implemented` is a `todo!()` you haven't replaced yet.
- `byte index N is out of range` or `is not a char boundary` is a slicing
  bug in your position tracking (R2).
- `"x" is not a slice of the input (zero-copy violated)` means you returned
  text from somewhere other than the input (R16). The usual causes are
  `to_string()`, `String::from`, `format!`, or a `""` literal for an empty field.

To see `println!` output from your parser while a test runs:

```
cargo test s02 -- --nocapture
```

### How to trust the tests

Before you rely on a test, read it once and ask: "Which rule does this
check, and does the expected value really follow from that rule?" Tests
can be wrong. If you find a test that contradicts this spec, fix the test
or the spec on purpose, not just to make it pass.

### Last step: the lifetime check

In `s08`, there is a commented-out test, `s08_rows_cannot_outlive_input`.
Uncomment it. `cargo test` must now fail to compile with
`error[E0597]: `input` does not live long enough`. If it compiles, your
lifetimes are wrong (R18). Comment it out again afterwards.
