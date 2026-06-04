## Generic stack

Implement a generic `Stack` struct backed by a `Vec`.

Add:

- `push()`
- `pop() -> Option<T>`
- `peek() -> Option<&T>`
- `is_empty() -> bool`
- `size() -> usize`

Test with `Stack<i32>` and `Stack<String>`.

**Hint:**

```rust
struct Stack<T> {
    data: Vec<T>,
}
```

For `peek`, return:

```rust
Some(&self.data[self.data.len() - 1])
```

### Get guidance

```text
struct backed by a Vec. Add push(), pop() -> Option, peek() -> Option<&T>, is_empty() -> bool, size() -> usize. Test with Stack and Stack.
```
