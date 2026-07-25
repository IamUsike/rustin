# Linked list with Box

Implement a singly linked list using an enum:

```rust
enum List {
    Cons(T, Box<Self>),
    Nil,
}

```

Add the following methods:

```
push_front()
len() -> usize
contains(&T) -> bool
to_vec() -> Vec<T>
```
