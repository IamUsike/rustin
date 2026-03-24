**Challenge 13: First Word Extractor [Medium]**

Implement `first_word(s: &str) -> &str` that returns a slice of the first word. Handle empty strings. Then extend it to `nth_word(s: &str, n: usize) -> Option<&str>` that returns the nth word.

**Hints:** find the first space with `.find(' ')` • use `split_whitespace().nth(n)`
