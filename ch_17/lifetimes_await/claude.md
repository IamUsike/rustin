# Code Review — Lifetimes Across `.await` and `tokio::spawn`

## TL;DR verdict

Your instincts are pointed in the right direction, but two spots need correcting:

1. **You correctly sensed** that `tokio::spawn` is different from a plain `.await` because the spawned task is detached and can outlive the caller's frame — that's exactly why `'static` is required.
2. **You mis-explained** why `soja(s).await` compiles inside `spawn` but the inline `println!` version doesn't. It's not about "the async fn getting cleaned up" — it's about **how the closure/async-block capture analysis treats the two different usages of `s`**.
3. **Minor misconception**: "`'static` means the data lives for as long as the program" is only true for _literal_ `'static` data (string literals, leaked memory). As a _bound_ (`T: 'static`), it just means "T owns everything it needs / holds no borrowed references shorter than the whole program" — an owned `String` or `Arc<str>` satisfies `T: 'static` even though the actual value can be dropped the next instant.

Details below, then the full exercise with three fixes.

---

## 1. Why `soja(s).await` works under `spawn`, but the inline block doesn't

This is the good question buried in your comments, and it's a genuinely subtle bit of Rust:

```rust
tokio::spawn(async { soja(s).await });          // compiles
tokio::spawn(async {
    println!("{s}");
    sleep(Duration::from_secs(3)).await;
    println!("{s}");
});                                               // "may outlive the current function, borrows `s`"
```

Neither block has `move`, so in both cases the compiler decides _how_ to capture `s` by looking at how it's **used** inside the block. That's the RFC 2229 "disjoint closure capture" analysis, and it applies to async blocks too since they desugar the same way closures do.

- **`println!("{s}")`** internally just needs `&s` (a shared reference, for `Display::fmt`). Since a borrow is all that's required, the compiler captures `s` **by reference** — i.e. it stores a reference to the _local variable_ `s` sitting on `main`'s stack. That reference's lifetime is tied to `main`'s stack frame. A `spawn`ed task can outlive that frame, so the compiler correctly refuses: the future is not `'static`.

- **`soja(s)`** — `soja` takes its parameter _by value_ (`s: &str` is a value parameter, even though the value itself happens to be a reference). To call it, the block needs to hand over an _owned_ `&str`, not a borrow of the local variable. Because `&str` is `Copy`, the compiler satisfies this by capturing `s` **by value** — it copies the pointer+length pair into the async block's own state at the moment the block is created. That copy is a plain `&'static str` (since your string literal is `'static` data), completely decoupled from `main`'s stack frame. A `'static` copy of a `'static` reference is happily `'static` — so `spawn` accepts it.

So the difference isn't "function calls are magic" — it's that **passing something by value to a function forces a by-value capture**, while **using it through a reference (println, method taking `&self`, etc.) forces a by-reference capture**. For non-`Copy` types this distinction usually needs to be made explicit with `move`; for `Copy` types like `&str`, the compiler can silently do the by-value capture without invalidating the original binding, which is also the answer to your last question:

## 2. Why you can still print `s` after "moving" it into the task

```rust
let task = tokio::spawn(async move {
    println!("task ... {s}");
    ...
});
let _ = tokio::join!(task);
println!("{s}"); // still works — why?
```

`move` transfers ownership of _whatever is captured_ into the closure/async block. For a `Copy` type, "moving" is indistinguishable from copying — the original `s` in `main` is never invalidated, because Copy types don't have the notion of a single owner whose absence you'd need to track. The `async move` block gets its own copy of the `&'static str`; `main`'s `s` binding is completely untouched. This is the exact same reason you can write:

```rust
let x = 5;
let f = move || x + 1;
println!("{x}"); // fine, i32 is Copy
```

If `s` were a `String` instead of `&str`, `async move` would genuinely move it, and the `println!("{s}")` after the join would fail with "value borrowed after move" (well, "used after move") — because `String` isn't `Copy`.

## 3. On your Arc/Mutex/'static notes

- **Arc** — correct that it's for sharing ownership across tasks. One nuance: you only need `Mutex`/`RwLock` _alongside_ it if something needs to **mutate** the shared data. Read-only shared data (e.g. a config loaded once) is fine as plain `Arc<T>` with no lock at all.
- **`'static`** — the common trap: `'static` as a _trait bound_ (`T: 'static`, which is what `tokio::spawn` requires) does **not** mean "the value lives until the program ends." It means "T contains no borrowed data with a lifetime shorter than the whole program" — i.e., T is either owned outright (`String`, `Vec<T>`, `Arc<T>`) or is/contains only genuinely `'static` references (string literals, `Box::leak`ed data, etc.). An owned `String` satisfies `T: 'static` and can be dropped the very next line. Only the _literal_ lifetime `'static` (as in `&'static str`) means "this specific reference points at data that truly lives for the whole program."

---

## 4. The exercise — reference across `.await`, and three fixes

```rust
use std::sync::Arc;
use tokio::time::{sleep, Duration};

// An async fn borrowing a &str, doing async work, then using it again after the await.
// This compiles fine as a plain, non-spawned call — the borrow never needs to
// outlive the caller's own stack frame.
async fn greet_slowly(name: &str) {
    println!("going to sleep with {name}");
    sleep(Duration::from_secs(1)).await;
    println!("finished sleeping with {name}"); // using the reference again after .await — fine here
}

#[tokio::main]
async fn main() {
    let owned = String::from("Ferris");

    // Works: runs to completion within main's own frame, so borrowing owned is safe.
    greet_slowly(&owned).await;

    // Does NOT compile if you try to spawn it directly:
    //
    //   tokio::spawn(greet_slowly(&owned));
    //
    // error[E0521]: borrowed data escapes outside of function
    //   `owned` does not live long enough — the future may outlive `main`,
    //   but it borrows `owned`, which is owned by the current function.
    //
    // Why: tokio::spawn hands the future to the runtime as an independent,
    // detachable task. The compiler can't prove the task finishes before
    // `owned` (and main's frame) goes away — even if you immediately
    // `.await` the JoinHandle, the *type* JoinHandle<T> demands T: 'static
    // unconditionally, because spawn has no way to special-case "but I
    // promise to join it right away."

    // ---------- Fix 1: clone the data ----------
    // Simplest option. Good when the data is small/cheap to duplicate and
    // each task doesn't need to see mutations made by others.
    // Downside: real duplication — O(n) cost and separate copies diverge
    // if you ever did need shared mutable state.
    let owned_clone = owned.clone();
    let t1 = tokio::spawn(async move {
        greet_slowly(&owned_clone).await;
    });

    // ---------- Fix 2: Arc<str> (shared ownership) ----------
    // Good when several tasks need read access to the *same* data without
    // duplicating it, especially if the data is large. Cloning an Arc is
    // just a refcount bump (cheap), not a deep copy.
    // Downside: slight overhead (atomic refcounting), and if you ever need
    // to mutate the shared value you must add a Mutex/RwLock on top —
    // Arc alone only gives you shared *read* access.
    let shared: Arc<str> = Arc::from(owned.as_str());
    let shared_for_task = Arc::clone(&shared);
    let t2 = tokio::spawn(async move {
        greet_slowly(&shared_for_task).await;
    });

    // ---------- Fix 3: genuinely 'static data ----------
    // Good for constants, config baked into the binary, or anything that
    // really should live for the whole program. String literals are
    // 'static automatically; Box::leak is the escape hatch for turning a
    // runtime-computed String into 'static (intentionally never freed).
    // Downside: Box::leak is a real, permanent memory leak — only reach
    // for it for one-time startup data, never in a loop or hot path.
    let literal: &'static str = "Ferris (static)";
    let t3 = tokio::spawn(async move {
        greet_slowly(literal).await;
    });

    let leaked: &'static str = Box::leak(owned.clone().into_boxed_str());
    let t4 = tokio::spawn(async move {
        greet_slowly(leaked).await;
    });

    let _ = tokio::join!(t1, t2, t3, t4);

    // owned is still usable here — Fix 1/2/3 all left it alone
    // (only borrowed from, or cloned before moving the clone).
    println!("original still here: {owned}");
}
```

### Trade-off summary

| Fix                                       | Cost                                       | When to reach for it                                    |
| ----------------------------------------- | ------------------------------------------ | ------------------------------------------------------- |
| `.clone()`                                | Real duplication (O(n))                    | Small/cheap data, no need to share mutations            |
| `Arc<T>` (+ `Mutex`/`RwLock` if mutating) | Refcount bump, or lock overhead if mutable | Larger data, genuinely shared across tasks              |
| `'static` literal / `Box::leak`           | Permanent leak (for `leak`)                | Truly program-lifetime constants, one-time startup data |
