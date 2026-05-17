CSV parser with Result

Write fn parse_csv(input: &str) -> Result<(), String> that parses a multi-line CSV string. Return Err if any row has a different number of columns than the first row.

Hint: Split on '\n' for rows, ',' for fields. Check field count after the first row.
