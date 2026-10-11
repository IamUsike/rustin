// Unit tests for CsvParser. See SPEC.md for the rules these tests enforce.
//
// Tests are grouped into stages (s01_, s02_, ...). Build the parser one stage
// at a time and run only that stage with:
//
//     cargo test s01
//
// When a stage is green, move to the next one. Earlier stages must stay green.

use super::*;
use std::collections::HashMap;

// ---------- helpers ----------

/// Parse `input` and collect every data row (header row excluded).
fn rows(input: &str) -> Vec<Vec<&str>> {
    CsvParser::new(input).collect()
}

/// Panic unless `s` points into the memory of `input`.
/// This is the zero-copy check: equal text is not enough, the bytes must be
/// the *same* bytes as the original input.
fn assert_borrowed(input: &str, s: &str) {
    let start = input.as_ptr() as usize;
    let end = start + input.len();
    let p = s.as_ptr() as usize;
    assert!(
        p >= start && p + s.len() <= end,
        "{s:?} is not a slice of the input (zero-copy violated)"
    );
}

// ---------- s01: headers ----------

#[test]
fn s01_headers_basic() {
    let p = CsvParser::new("name,age\n");
    assert_eq!(p.headers(), &["name", "age"]);
}

#[test]
fn s01_headers_without_trailing_newline() {
    let p = CsvParser::new("name,age");
    assert_eq!(p.headers(), &["name", "age"]);
}

#[test]
fn s01_header_only_input_has_no_rows() {
    assert_eq!(rows("name,age\n"), Vec::<Vec<&str>>::new());
    assert_eq!(rows("name,age"), Vec::<Vec<&str>>::new());
}

#[test]
fn s01_empty_input() {
    let p = CsvParser::new("");
    dbg!(&p);
    assert!(p.headers().is_empty());
    assert_eq!(rows(""), Vec::<Vec<&str>>::new());
}

#[test]
fn s01_single_column_header() {
    let p = CsvParser::new("id\n");
    assert_eq!(p.headers(), &["id"]);
}

// ---------- s02: one data row ----------

#[test]
fn s02_one_row() {
    assert_eq!(rows("a,b\n1,2\n"), vec![vec!["1", "2"]]);
}

#[test]
fn s02_one_row_without_trailing_newline() {
    assert_eq!(rows("a,b\n1,2"), vec![vec!["1", "2"]]);
}

#[test]
fn s02_single_column_row() {
    assert_eq!(rows("id\n42\n"), vec![vec!["42"]]);
}

// ---------- s03: many rows ----------

#[test]
fn s03_many_rows() {
    let input = "a,b\n1,2\n3,4\n5,6\n";
    assert_eq!(
        rows(input),
        vec![vec!["1", "2"], vec!["3", "4"], vec!["5", "6"]]
    );
}

#[test]
fn s03_many_rows_without_trailing_newline() {
    assert_eq!(rows("a,b\n1,2\n3,4"), vec![vec!["1", "2"], vec!["3", "4"]]);
}

#[test]
fn s03_next_returns_none_forever_after_end() {
    let mut p = CsvParser::new("a\n1\n");
    assert_eq!(p.next(), Some(vec!["1"]));
    assert_eq!(p.next(), None);
    assert_eq!(p.next(), None);
    assert_eq!(p.next(), None);
}

// ---------- s04: empty fields ----------

#[test]
fn s04_empty_last_field() {
    assert_eq!(rows("a,b\n1,\n"), vec![vec!["1", ""]]);
}

#[test]
fn s04_empty_first_field() {
    assert_eq!(rows("a,b\n,2\n"), vec![vec!["", "2"]]);
}

#[test]
fn s04_all_fields_empty() {
    assert_eq!(rows("a,b,c\n,,\n"), vec![vec!["", "", ""]]);
}

#[test]
fn s04_empty_field_at_end_of_input() {
    assert_eq!(rows("a,b\n1,"), vec![vec!["1", ""]]);
}

#[test]
fn s04_empty_header_name() {
    let p = CsvParser::new("a,,c\n");
    assert_eq!(p.headers(), &["a", "", "c"]);
}

// ---------- s05: CRLF line endings ----------

#[test]
fn s05_crlf_rows() {
    assert_eq!(
        rows("a,b\r\n1,2\r\n3,4\r\n"),
        vec![vec!["1", "2"], vec!["3", "4"]]
    );
}

#[test]
fn s05_crlf_headers_have_no_carriage_return() {
    let p = CsvParser::new("a,b\r\n1,2\r\n");
    assert_eq!(p.headers(), &["a", "b"]);
}

#[test]
fn s05_crlf_empty_last_field() {
    assert_eq!(rows("a,b\r\n1,\r\n"), vec![vec!["1", ""]]);
}

#[test]
fn s05_mixed_line_endings() {
    assert_eq!(
        rows("a,b\r\n1,2\n3,4\r\n"),
        vec![vec!["1", "2"], vec!["3", "4"]]
    );
}

#[test]
fn s05_lone_carriage_return_is_ordinary_data() {
    // Only "\r\n" is a line ending. A "\r" not followed by "\n" is data.
    assert_eq!(rows("a,b\n1\r2,3\n"), vec![vec!["1\r2", "3"]]);
}

// ---------- s06: blank lines ----------

#[test]
fn s06_blank_lines_between_rows_are_skipped() {
    assert_eq!(
        rows("a,b\n1,2\n\n3,4\n"),
        vec![vec!["1", "2"], vec!["3", "4"]]
    );
}

#[test]
fn s06_blank_lines_at_end_are_skipped() {
    assert_eq!(rows("a,b\n1,2\n\n\n"), vec![vec!["1", "2"]]);
}

#[test]
fn s06_blank_line_after_header_is_skipped() {
    assert_eq!(rows("a,b\n\n1,2\n"), vec![vec!["1", "2"]]);
}

#[test]
fn s06_blank_crlf_lines_are_skipped() {
    assert_eq!(rows("a,b\r\n\r\n1,2\r\n\r\n"), vec![vec!["1", "2"]]);
}

#[test]
fn s06_only_blank_lines_after_header() {
    assert_eq!(rows("a,b\n\n\n"), Vec::<Vec<&str>>::new());
}

// ---------- s07: things the parser must NOT do (see SPEC.md "Out of scope") ----------

#[test]
fn s07_quotes_are_ordinary_characters() {
    assert_eq!(rows("a,b\n\"x\",y\n"), vec![vec!["\"x\"", "y"]]);
}

#[test]
fn s07_comma_inside_quotes_still_splits() {
    // No quoting support: the comma is a delimiter even between quotes.
    assert_eq!(rows("a,b\n\"x,y\"\n"), vec![vec!["\"x", "y\""]]);
}

#[test]
fn s07_whitespace_is_kept() {
    assert_eq!(rows("a,b\n 1 , 2 \n"), vec![vec![" 1 ", " 2 "]]);
}

#[test]
fn s07_ragged_rows_are_yielded_as_is() {
    assert_eq!(
        rows("a,b\n1\n1,2,3\n"),
        vec![vec!["1"], vec!["1", "2", "3"]]
    );
}

#[test]
fn s07_unicode_fields() {
    let p = CsvParser::new("名前,年齢\n太郎,20\n");
    assert_eq!(p.headers(), &["名前", "年齢"]);
    assert_eq!(rows("名前,年齢\n太郎,20\n"), vec![vec!["太郎", "20"]]);
}

// ---------- s08: zero-copy ----------

#[test]
fn s08_headers_borrow_from_input() {
    let input = String::from("name,age\r\nann,30\n");
    let p = CsvParser::new(&input);
    for h in p.headers() {
        assert_borrowed(&input, h);
    }
}

#[test]
fn s08_fields_borrow_from_input() {
    let input = String::from("a,b,c\n1,,3\r\n\n4,5,6");
    for row in CsvParser::new(&input) {
        for field in row {
            assert_borrowed(&input, field);
        }
    }
}

// Lifetime check. Uncomment this test and run `cargo test`: it must FAIL TO
// COMPILE with "`input` does not live long enough" (or similar). If it
// compiles, your 'a annotations are not tying rows to the input. Comment it
// out again afterwards.
//
// #[test]
// fn s08_rows_cannot_outlive_input() {
//     let row;
//     {
//         let input = String::from("a\n1\n");
//         row = CsvParser::new(&input).next();
//     }
//     println!("{row:?}");
// }

// ---------- s09: record ----------

#[test]
fn s09_record_zips_headers_with_row() {
    let input = "name,age\nann,30\n";
    let mut p = CsvParser::new(input);
    let row = p.next().unwrap();
    let rec = p.record(row);
    let expected = HashMap::from([("name", "ann"), ("age", "30")]);
    assert_eq!(rec, expected);
}

#[test]
fn s09_record_short_row_drops_missing_columns() {
    let mut p = CsvParser::new("a,b,c\n1,2\n");
    let row = p.next().unwrap();
    assert_eq!(p.record(row), HashMap::from([("a", "1"), ("b", "2")]));
}

#[test]
fn s09_record_long_row_drops_extra_fields() {
    let mut p = CsvParser::new("a,b\n1,2,3\n");
    let row = p.next().unwrap();
    assert_eq!(p.record(row), HashMap::from([("a", "1"), ("b", "2")]));
}

#[test]
fn s09_record_borrows_from_input() {
    let input = String::from("name,age\nann,30\n");
    let mut p = CsvParser::new(&input);
    let row = p.next().unwrap();
    for (k, v) in p.record(row) {
        assert_borrowed(&input, k);
        assert_borrowed(&input, v);
    }
}

// ---------- s10: query ----------

const PEOPLE: &str = "name,city,age\n\
                      ann,paris,30\n\
                      bob,rome,25\n\
                      cat,paris,41\n";

#[test]
fn s10_query_returns_all_matches_in_order() {
    let result = query(PEOPLE, "city", "paris");
    assert_eq!(
        result,
        vec![
            HashMap::from([("name", "ann"), ("city", "paris"), ("age", "30")]),
            HashMap::from([("name", "cat"), ("city", "paris"), ("age", "41")]),
        ]
    );
}

#[test]
fn s10_query_single_match() {
    let result = query(PEOPLE, "name", "bob");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0]["age"], "25");
}

#[test]
fn s10_query_no_match() {
    assert!(query(PEOPLE, "city", "tokyo").is_empty());
}

#[test]
fn s10_query_unknown_column() {
    assert!(query(PEOPLE, "country", "paris").is_empty());
}

#[test]
fn s10_query_is_case_sensitive_and_exact() {
    assert!(query(PEOPLE, "city", "Paris").is_empty());
    assert!(query(PEOPLE, "city", "par").is_empty());
    assert!(query(PEOPLE, "City", "paris").is_empty());
}

#[test]
fn s10_query_empty_input() {
    assert!(query("", "city", "paris").is_empty());
}

#[test]
fn s10_query_can_match_empty_value() {
    let result = query("a,b\n1,\n2,x\n", "b", "");
    assert_eq!(result, vec![HashMap::from([("a", "1"), ("b", "")])]);
}

#[test]
fn s10_query_results_borrow_from_input_not_from_arguments() {
    let input = String::from(PEOPLE);
    // `column` and `value` are separate Strings that are dropped below.
    // Results must point into `input`, never into them.
    let column = String::from("city");
    let value = String::from("paris");
    let result = query(&input, &column, &value);
    drop(column);
    drop(value);
    for rec in &result {
        for (k, v) in rec {
            assert_borrowed(&input, k);
            assert_borrowed(&input, v);
        }
    }
}
