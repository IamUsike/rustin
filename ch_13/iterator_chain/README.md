### Iterator chain pipeline

Given a Vec of strings representing mixed data (`"42"`, `"foo"`, `"17"`, `"bar"`, `"100"`), use a single iterator chain to:

- filter valid i32s,
- double them,
- keep only those > 50,
- collect into a Vec.

No loops.

**Hint:** Use `.iter().filter_map(|s| s.parse().ok()).map(|n| n * 2).filter(|&n| n > 50).collect()`

---

Iterator adapter
is a method that:

1.  Takes an iterator
2.  Wraps under some behaviour
3.  Returns another iterator

eg:

```rust
.iter()
.map(...)
.filter(...)
.filter_map(...)
.take(...)
.skip(...)
```

All these return new iterators.

Think of it like building a pipeline

```
Vec
 ↓
iter()
 ↓
filter_map(...)
 ↓
map(...)
 ↓
filter(...)
```

At this point nothing is processed yet, just describing what'll happen later.

Eg:

```rust
let nums = vec![1, 2, 3];

let doubled = nums.iter().map(|n| {
    println!("doubling {}", n);
    n * 2
});
```

Nothing gets printed here cos the closure hasnt run yet
`map()` just created a `Map` iterator adapter

this runs when something consumes this iterator. Like:

```rust
collect()
for_each()
count()
sum()
find()
```

eg:

```rust
let nums = vec![1, 2, 3];

let doubled = nums.iter().map(|n| {
    println!("doubling {}", n);
    n * 2
});

let result: Vec<_> = doubled.collect();
```
