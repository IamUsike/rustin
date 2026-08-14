## Parallel map

Write a function `parallel_map(data: Vec<i32>, f: fn(i32) -> i32) -> Vec<i32>` that applies f to each element using one thread per element. The output Vec must be in the same order as the input — not whatever order threads finish. Think about how to preserve order before writing any code.

> Cements: threads don't finish in order, you must account for that explicitly
