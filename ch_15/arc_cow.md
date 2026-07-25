### Arc

```
pub struct Arc<T, A = Global>where
    A: Allocator,
    T: ?Sized,{ /* private fields */ }
```

arc is a struct over the generics A and T. T can be sized or unsized.

arc comes in concurrency

---

## Enum Cow

- A Clone on Write smart pointer.
- `Cow` implements `Deref` which means that you can call non-mutating methods directly on the data it encloses.
