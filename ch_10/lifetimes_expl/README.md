# Annotate, then explain every lifetime

Write these 5 functions WITHOUT lifetime elision — explicit annotations everywhere, even where the compiler would infer them. For each one, write a comment explaining in plain English what the lifetime annotation means: (1) fn first_word(s: &str) -> &str. (2) fn longer<'a>(s1: &'a str, s2: &'a str) -> &'a str. (3) fn first<'a, T>(slice: &'a [T]) -> Option<&'a T>. (4) struct StrSplit<'a> { remainder: &'a str }. (5) fn substr<'a>(s: &'a str, start: usize, len: usize) -> &'a str.

> What this cements: Lifetimes aren't about how long something lives — they're about relationships. 'a on input and output means "output can't outlive this input". Writing them explicitly makes elision feel like a reward, not a mystery.
