fn parse_csv(input: &str) -> Result<(), String> {
    // Step 1:
    // Split input into lines and IGNORE empty/whitespace-only lines
    // This avoids treating blank lines as invalid rows (which would break column count)
    let mut lines = input.lines().filter(|line| !line.trim().is_empty());

    // Step 2:
    // Safely get the first row
    // If there are no rows (empty input), we consider it valid
    let first = match lines.next() {
        Some(line) => line,
        None => return Ok(()), // empty CSV → valid
    };

    // Step 3:
    // Determine expected number of columns using the first row
    // We just COUNT splits instead of storing them (more efficient)
    let expected_len = first.split(',').count();

    // Step 4:
    // Iterate over remaining rows and compare column counts
    // enumerate() gives (index, line)
    // index starts from 0 → corresponds to second row
    for (i, line) in lines.enumerate() {
        let count = line.split(',').count();

        // If any row has a different number of columns → error
        if count != expected_len {
            return Err(format!(
                "Row {} has {} columns, expected {}",
                i + 2, // +2 because:
                // i = 0 → second row
                // i = 1 → third row, etc.
                count,
                expected_len
            ));
        }
    }

    // Step 5:
    // If all rows matched → valid CSV
    Ok(())
}

/*
==========================
⚠️ Important Edge Cases
==========================

1. Empty input:
   "" → Ok(())

2. Empty lines:
   a,b,c
   1,2,3

   (blank line ignored)

3. Trailing commas:
   a,b,c
   1,2,3,

   → "1,2,3," splits into ["1","2","3",""]
   → column count = 4 ❌ (mismatch)

4. Spaces:
   "a, b, c" → spaces are part of values (not trimmed automatically)

5. Only commas:
   ",," → ["", "", ""] → 3 columns (valid if consistent)

==========================
🧠 Key Idea
==========================

- Use first row as reference
- Stream through remaining rows (no need to store everything)
- Fail fast on mismatch
- Ignore empty lines for robustness
*/
