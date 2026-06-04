## Lifetime annotations

Write `fn longest_starting_with<'a>(s1: &'a str, s2: &'a str, prefix: &str) -> &'a str` that returns whichever of `s1` or `s2` is longer, but only if it starts with prefix. If neither qualifies, return s1. Add explicit lifetime annotations everywhere and explain each in a comment.

Hint: The return must be tied to the input lifetimes. Return type &'a str means it can't outlive either input.

```text
(s1: &a str, s2: &a str, prefix: &str) -> &a str that returns whichever of s1 or s2 is longer, but only if it starts with prefix. If neither qualifies, return s1. Add explicit lifetime annotations everywhere and explain each in a comment.
```
