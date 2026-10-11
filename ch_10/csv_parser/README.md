# Build a zero-copy CSV reader

Build CsvReader<'a> that takes a &'a str of CSV data and lazily yields rows as Vec<&'a str> — no allocations for the strings themselves, every &str is a slice of the original input. Implement Iterator for it. Add: fn headers(&self) -> &[&'a str] (first row), fn record(row: Vec<&'a str>) -> HashMap<&'a str, &'a str> (zip headers with a row). Write a fn query<'a>(csv: &'a str, column: &str, value: &str) -> Vec<HashMap<&'a str, &'a str>> that finds all rows where column == value. Every returned string must borrow from the original csv input — zero heap allocation for string data.

> What this cements: This is real systems programming. Zero-copy parsing is how high-performance parsers (nom, httparse, Suricata's Rust parsers) work. The entire challenge is threading the 'a lifetime correctly so the compiler can verify no string outlives the input buffer.
