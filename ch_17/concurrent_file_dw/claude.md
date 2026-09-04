# Concurrent File Downloader (JoinSet) — Code Review

## TL;DR

| #   | Thing                                                                                                 | Verdict                                                                                                                                                                                                                    |
| --- | ----------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | `res.unwrap()` on line inside the loop                                                                | **Won't compile.** `res` is already a `String` (already unwrapped by the `Some(Ok(res))` pattern) — `String` has no `.unwrap()` method                                                                                     |
| 2   | `while let Some(Ok(res)) = tasks.join_next().await`                                                   | **Real bug**, not just style: if _any_ task panics, `join_next()` returns `Some(Err(_))`, the pattern fails to match, and the `while let` **exits the loop entirely** — you silently lose every remaining in-flight result |
| 3   | `let task = tasks.spawn(...)`                                                                         | Unused-variable warning — you never use the returned `AbortHandle`                                                                                                                                                         |
| 4   | `size_kb: u64` in `download`, but tuples use plain int literals                                       | Compiles fine (type inference unifies the literals to `u64`), but the exercise asked for `u32` — worth matching the spec explicitly rather than relying on inference                                                       |
| 5   | Your inline question: _"can I `tokio::spawn` separately and feed the `JoinHandle` into a `JoinSet`?"_ | **No** — `JoinSet` has no API to adopt an existing `JoinHandle`. You spawn directly _onto_ the `JoinSet` via `.spawn()`, which is exactly what you did — that part's correct                                               |
| 6   | Stale comment `//check Option<Option<&str>> later`                                                    | Leftover, doesn't match final code — safe to delete                                                                                                                                                                        |
| 7   | Overall structure (spawn 6 onto `JoinSet`, drain with `join_next().await` in a loop)                  | **Correct approach** for "process results as they complete, not in submission order" — this is exactly what `JoinSet` is for                                                                                               |

---

## 1. Why it won't compile

```rust
while let Some(Ok(res)) = tasks.join_next().await {
    let output = res.unwrap();   // <-- res is already a String here
    ...
}
```

`JoinSet<String>::join_next()` returns `Option<Result<String, JoinError>>`. Once you destructure with the pattern `Some(Ok(res))`, `res` is already bound as the plain `String` — the `Ok(...)` in the pattern _is_ the unwrap. Calling `.unwrap()` on it again is calling a method that doesn't exist on `String`:

```
error[E0599]: no method named `unwrap` found for struct `String` in the current scope
```

Fix: just use `res` directly.

## 2. The silent-truncation bug

This is the more important one, because it _does_ compile once you fix #1 — it just does the wrong thing under a specific condition:

```rust
while let Some(Ok(res)) = tasks.join_next().await { ... }
```

`while let PATTERN = EXPR { ... }` only continues looping while the pattern keeps matching. `join_next()` yields `Some(Err(join_err))` if a task panicked (or was cancelled) — that's a valid, expected outcome, not the end of the set. But your pattern only matches `Some(Ok(_))`, so the very first panicked task causes the whole `while let` to treat it like `None` (the "set is empty" case) and **stop polling the JoinSet altogether** — even if 3 other downloads are still in flight. You won't see an error and you won't see the remaining results; they just vanish.

Fix: match on the full `Option<Result<_, _>>` and only stop when you actually get `None`:

```rust
while let Some(res) = tasks.join_next().await {
    match res {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("task failed: {e}"),
    }
}
```

## 3. Your inline question, answered

> Is it possible to send a `JoinHandle` (from `tokio::spawn`) into a `JoinSet`, and use the `JoinSet` just to poll whichever finishes first?

No — `tokio::task::JoinSet` doesn't expose any method to adopt an already-existing `JoinHandle`. The only way to get a task into a `JoinSet` is to spawn it _through_ the set itself (`JoinSet::spawn`, `spawn_local`, `spawn_blocking`, etc.) — the set owns the handles internally and that's what lets `join_next()` do "whichever finishes first" polling across all of them. So what you wrote (`tasks.spawn(...)` for each URL, then draining with `join_next()`) is the correct and idiomatic pattern — no changes needed there.

(For contrast: if you only had loose `JoinHandle`s from separate `tokio::spawn` calls and wanted "first one ready," you'd reach for `futures::future::select_all` instead — but that's a different, clunkier tool than `JoinSet` for this exact job.)

---

## 4. Fixed version

```rust
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let urls: Vec<(&str, u32)> = vec![
        ("one", 120),
        ("two", 40),
        ("three", 280),
        ("four", 200),
        ("five", 160),
        ("six", 80),
    ];

    let mut tasks: JoinSet<String> = JoinSet::new();

    // Spawn directly onto the JoinSet. There's no way to hand an existing
    // JoinHandle to a JoinSet after the fact — spawning through .spawn() is
    // how tasks get registered with it in the first place.
    for (url, size_kb) in urls {
        tasks.spawn(async move { download(url, size_kb).await });
        // no `let` binding needed — we don't need the AbortHandle here
    }

    // Drain the set as tasks complete, in COMPLETION order, not submission
    // order. join_next() returns None only once every task has been polled
    // out — so we must handle Err (panicked/cancelled task) explicitly
    // instead of letting the pattern match silently end the loop early.
    while let Some(res) = tasks.join_next().await {
        match res {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("a download task failed: {e}"),
        }
    }
}

async fn download(url: &str, size_kb: u32) -> String {
    println!("starting download: {url}");
    sleep(Duration::from_millis(size_kb as u64 * 10)).await;
    format!("{url} downloaded")
}
```

Expected completion order (by simulated transfer time, ascending):
`two (400ms) -> six (800ms) -> one (1200ms) -> five (1600ms) -> four (2000ms) -> three (2800ms)`

— i.e. NOT the `one, two, three, four, five, six` submission order, which is the whole point of using `JoinSet::join_next()` over `join_all` on a `Vec<JoinHandle>` (which would still let you await all 6, but wouldn't let you _react_ to each one the moment it's ready without extra bookkeeping).
