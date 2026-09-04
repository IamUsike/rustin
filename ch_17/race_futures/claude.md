# `select!` Race & Timeout — Code Review

## TL;DR

| #   | Thing                                                                                                                                                                 | Verdict                                                                                                                                                                                                                                                                                        |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Your comment: _"creating another async fn instead of spawning would be better cos exec starts when select polls it, unlike spawn which starts immediately in the bg"_ | **You're completely right** — but then the actual code doesn't apply it (see #2)                                                                                                                                                                                                               |
| 2   | `tokio::spawn(async { ... sleep(5000)... })` passed into `fetch`                                                                                                      | Contradicts the exercise's "cancel the other" requirement. A spawned task starts running on the runtime immediately and keeps running to completion in the background **regardless of `select!`** — the loser isn't cancelled, you just stop _waiting_ on it                                   |
| 3   | Your comment: _"check why the message is `Result<>` instead of `T::Output`"_                                                                                          | Correct catch, and here's why: `T = JoinHandle<String>`, and `JoinHandle<T>::Output` is `Result<T, JoinError>` — **not** `T`. So `Result<T::Output, String>` is really `Result<Result<String, JoinError>, String>`, a nested `Result` you didn't intend                                        |
| 4   | `Err(message) => { println!("Failed with 'Message'"); }`                                                                                                              | Bug — hardcoded literal string, `message` is never actually printed (also triggers an unused-variable warning)                                                                                                                                                                                 |
| 5   | Commented-out two-task race version                                                                                                                                   | Same spawn issue as #2, plus: since both tasks are spawned _before_ `select!`, "cancellation" of the loser in that snippet only happens incidentally because `main()` returns and the whole runtime shuts down — not because `select!` cancelled anything                                      |
| 6   | Your 3 "Things to verify" notes at the bottom                                                                                                                         | All three are correct: (1) you must _call_ `sleep(...)` to produce a future, not reference the fn itself; (2) yes, a spawned task is already running independently, `select!` just observes it; (3) correct, `select!` polls the arm expressions internally — you never `.await` them yourself |
| 7   | `use tokio;`                                                                                                                                                          | Harmless but redundant — Rust 2018+ doesn't need this to use `tokio::spawn` etc. via full paths                                                                                                                                                                                                |

---

## 1. The core issue: spawn kills real cancellation

This is the crux of the whole exercise. Two different patterns look similar but mean very different things:

**Pattern A — raw future, no spawn (true cancellation):**

```rust
let work = async { sleep(Duration::from_millis(5000)).await; "done" };
tokio::select! {
    res = work => { /* work wins */ }
    _ = sleep(Duration::from_millis(100)) => { /* timeout wins */ }
}
// `work` is dropped right here if timeout won. It never started running
// independently, so dropping it BEFORE it completes genuinely stops it.
```

**Pattern B — spawned task (no real cancellation, just "stop waiting"):**

```rust
let handle = tokio::spawn(async { sleep(Duration::from_millis(5000)).await; "done" });
tokio::select! {
    res = handle => { /* handle wins */ }
    _ = sleep(Duration::from_millis(100)) => { /* timeout wins */ }
}
// The spawned task keeps running on the runtime to completion in the
// background even after select! returns — you just gave up awaiting its result.
```

Your code (and the commented-out version) both use **Pattern B**, which is why your own comment about "exec starts when select polls it, unlike spawn" is the right insight — you'd already spotted that spawning defeats the purpose, just hadn't applied it yet.

Pattern B isn't _wrong_ — it's a legitimate, different pattern ("fire off work, give up waiting after N ms, let it finish in the background") — but it doesn't satisfy "cancel the other," which is what the exercise asked for.

### If you _do_ want a spawned task cancelled on timeout

Sometimes you want the work on its own spawned task (e.g. it needs to keep running even if the caller is dropped, or you want it scheduled independently) **and** you want a real cancel on timeout. Tokio gives you `JoinHandle::abort()` for exactly this — poll `&mut handle` (not `handle`) so you retain ownership to call `.abort()` in the other branch:

```rust
let mut handle = tokio::spawn(async {
    sleep(Duration::from_millis(5000)).await;
    "done"
});

tokio::select! {
    res = &mut handle => {
        match res {
            Ok(v) => println!("finished: {v}"),
            Err(e) => println!("task panicked: {e}"),
        }
    }
    _ = sleep(Duration::from_millis(100)) => {
        handle.abort(); // <-- actually cancels the spawned task
        println!("timed out, aborted the task");
    }
}
```

---

## 2. The nested-`Result` bug, concretely

In your code, `sl_time = 100` and `task1` sleeps `5000`ms, so the timeout branch always wins in practice — which is exactly why you never _see_ the nested-`Result` problem manifest, even though it's latently there. If you flipped it (`sl_time = 100_000`, say), the `Ok` arm would fire and you'd see something like:

```
Succeeded with Ok("Task successfully completed")!
```

— note the extra `Ok(...)` wrapper. That's `T::Output` where `T = JoinHandle<String>` being `Result<String, JoinError>`, not `String`. The fix is simply: don't hand `fetch` a `JoinHandle` — hand it the raw future directly, so `T::Output` is the value you actually produced.

---

## 3. Fixed version — both parts of the exercise

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    basic_race().await;
    fetch_with_timeout_demo().await;
}

// ---------- Part 1: race two futures, loser gets genuinely dropped ----------
async fn basic_race() {
    let fast = async {
        sleep(Duration::from_millis(200)).await;
        "fast result"
    };
    let slow = async {
        sleep(Duration::from_millis(2000)).await;
        "slow result"
    };

    // Neither future has started running independently of this select! —
    // whichever branch doesn't win is simply dropped, un-polled to completion.
    tokio::select! {
        res = fast => println!("fast won: {res}"),
        res = slow => println!("slow won: {res}"),
    }
}

// ---------- Part 2: fetch-with-timeout, generic over the raw future ----------
// Note: bound is over a plain Future, NOT a JoinHandle, so T::Output is
// exactly the value your async work produces -- no nested Result.
async fn fetch_with_timeout<T: Future>(work: T, timeout_ms: u64) -> Result<T::Output, String> {
    tokio::select! {
        val = work => {
            println!("work completed before timeout");
            Ok(val)
        }
        _ = sleep(Duration::from_millis(timeout_ms)) => {
            println!("timed out");
            Err(String::from("timed out"))
        }
    }
}

async fn fetch_with_timeout_demo() {
    let work = async {
        println!("starting work");
        sleep(Duration::from_millis(5000)).await;
        String::from("data fetched successfully")
    };

    match fetch_with_timeout(work, 100).await {
        Ok(message) => println!("Succeeded with {message:?}!"),
        Err(message) => println!("Failed with {message:?}"), // now actually prints it
    }
}
```

`fetch_with_timeout`'s `work` future is dropped, un-run-to-completion, the instant the 100ms timeout wins — that's the real cancellation the exercise is after, and `Ok(message)` in the demo is a plain `String`, matching what you'd naturally expect from `T::Output`.
