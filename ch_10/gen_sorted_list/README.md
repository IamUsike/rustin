# Generic sorted list

Build a `SortedList` that maintains sorted order on every insert.

Methods:

- `insert(item: T)`
- `remove(&T) -> bool`
- `contains(&T) -> bool`
- `min() -> Option<&T>`
- `max() -> Option<&T>`
- `as_slice() -> &[T]`

**Hint:** Use `binary_search()` to find insertion position. Insert at that index with `Vec::insert(pos, item)`.
