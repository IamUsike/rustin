### Printable + summarize traits

Define a `Summarize` trait with `fn summary(&self) -> String`.

Implement it for:

- `Article` (`title`, `author`, `content`)
- `Tweet` (`username`, `content`)

Write a generic function:

```rust
fn print_summary(item: &T)
```

```rust
impl Summarize for Article {
    fn summary(&self) -> String {
        format!("({}, by {})", self.title, self.author)
    }
}
```
