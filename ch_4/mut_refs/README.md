**Challenge 12: Mutable References Puzzle [Medium]**

Write a function `double_all(v: &mut Vec<i32>)` that doubles every element in-place. Write another `sum_slice(s: &[i32]) -> i32`. Demonstrate that you cannot have a mutable and immutable borrow at the same time.

**Hints:** use `iter_mut()` for mutable iteration • show the borrow error in a comment, then fix it
