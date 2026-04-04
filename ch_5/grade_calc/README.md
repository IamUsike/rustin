## Student grade tracker (medium)

Define a `Student` struct with `name` (`String`) and `grades` (`Vec<f64>`).

Add methods:

- `average() -> f64`
- `highest() -> f64`
- `lowest() -> f64`
- `letter_grade() -> &str`

Create 3 students and print a summary table.

**Hint:**  
Use `f64::MIN` and `f64::MAX` as initial min/max values.  
For letter grade, match on average ranges.
