# Async Mutex & Deadlock — Code Review

## TL;DR

| #   | Thing                                                | Verdict                                                                                                                               |
| --- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | "Why do we need a Mutex at all?"                     | Mostly right — but it's **type-system + real parallelism**, not just "sometimes a different thread"                                   |
| 2   | "Why `tokio::sync::Mutex` prevents the issue"        | Right idea, slightly garbled explanation — see below                                                                                  |
| 3   | Does your code actually deadlock?                    | **No.** You already drop the guard before the `.await`, so there's nothing to deadlock on                                             |
| 4   | Loop comment ("spawn 5 tasks")                       | Off by one — `0..4` is 4 tasks, +1 separate task = 5 total. Comment is misleading in isolation                                        |
| 5   | `for _ in 1..1000` on the slow task                  | 999 iterations × 5s sleep = **~83 minutes** before `join_all` returns. This will look like a hang, but it's just math, not a deadlock |
| 6   | `println!("{:?}", ctr)`                              | Prints the `Mutex` debug wrapper, not the count. Works, but ugly — lock it and print the value                                        |
| 7   | The actual exercise (std Mutex held across `.await`) | **Never implemented** — your code always used `tokio::sync::Mutex`, so the buggy scenario was never exercised                         |

---

## 1. Your questions, answered

### Q: "Why do we need a mutex — is it because the runtime might run tasks on a different thread?"

Yes, that's _part_ of it, and the more important part is one you didn't mention:

- `#[tokio::main]` defaults to the **multi-threaded** runtime (one worker thread per CPU core). Tasks spawned with `tokio::spawn` really can run **in parallel**, on different OS threads, at the same instant. So `*ctr += 1` (a read-modify-write) is a genuine data race without synchronization.
- Independent of threading: `Arc<T>` only ever gives you `&T` (shared reference), never `&mut T`. Rust's borrow checker won't let you mutate through a shared reference at all. So even on a hypothetical single-threaded, purely cooperative scheduler, you'd _still_ need some interior-mutability type (`Mutex`, `RwLock`, atomics, `RefCell` if single-threaded) just to satisfy the compiler — the Mutex isn't only about races, it's your only legal path to mutate shared state through an `Arc`.

So: it's both "the runtime might genuinely run this on another core" _and_ "the type system requires it regardless."

### Q: "Why does `tokio::sync::Mutex` fix the std-Mutex-across-await problem?"

Your explanation is directionally correct but let's tighten it up:

- `std::sync::Mutex::lock()` is a **blocking, synchronous** call. If the lock is contended, the calling thread physically parks/spins at the OS level until it's free. It does **not** know how to yield to the async executor — it just blocks whatever OS thread it's running on.
- `tokio::sync::Mutex::lock()` returns a `Future`. If contended, `.await`-ing it **suspends the task** (registers a waker) and hands the OS thread back to the executor, which goes and runs other ready tasks. No OS thread is blocked.

The actual danger scenario (which your code never triggers, see §3) is:

1. Task A does `std_mutex.lock()`, gets the guard.
2. Task A hits an `.await` (e.g. `sleep`) **while still holding that guard**.
3. Task A is suspended; its OS thread is freed up and may go run Task B.
4. Task B calls `std_mutex.lock()` — contended, so it **blocks its OS thread** synchronously waiting for A to unlock.
5. If enough worker threads end up blocked this way (or you're on a single-threaded runtime), nobody is left to ever poll Task A again to finish its sleep and drop the guard → **deadlock / total stall**.

---

## 2. Why your code does NOT actually deadlock

Look at your "slow" task:

```rust
{
    let mut ctr = ctr.lock().await;
    *ctr += 1;
}                              // <- guard dropped HERE, at end of block
sleep(Duration::from_secs(5)).await;   // <- lock is already released by this point
```

You explicitly scoped the guard in its own `{ }` block so it drops **before** the `sleep().await`. That's actually the _correct_, safe pattern — you accidentally wrote the fix, not the bug. Combined with `tokio::sync::Mutex` (which is safe to hold across `.await` anyway), there is genuinely no deadlock possible here. Your inline comment ("locks the ctr for 1k secs... creates a deadlock") describes a real hazard in the abstract, but it doesn't apply to the code above it.

Also — your `let t = drop(...)` comment about `Send` is a real clue: that compiler error is exactly what you get when you try to hold a `std::sync::MutexGuard` across an `.await` in code that gets passed to `tokio::spawn`. More on that below.

---

## 3. The actual buggy version (what the exercise wants)

This is what you need to _deliberately_ write to see the failure mode:

```rust
use std::sync::{Arc, Mutex}; // <-- std, not tokio::sync — this is the bug
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let ctr = Arc::new(Mutex::new(0));

    let ctr_a = Arc::clone(&ctr);
    let t1 = tokio::spawn(async move {
        let mut guard = ctr_a.lock().unwrap(); // sync (blocking) lock, not .await'd
        *guard += 1;
        // BUG: guard is still alive here, and we now hit an .await while holding it
        sleep(Duration::from_secs(5)).await;
        // guard finally drops here, at end of scope
    });

    let ctr_b = Arc::clone(&ctr);
    let t2 = tokio::spawn(async move {
        let mut guard = ctr_b.lock().unwrap(); // wants the same lock
        *guard += 1;
    });

    let _ = tokio::join!(t1, t2);
}
```

What actually happens when you try this:

- **On the default multi-threaded runtime (what you have via `#[tokio::main]`)**: this **won't even compile**. `tokio::spawn` requires the spawned future to be `Send`. `std::sync::MutexGuard` is explicitly `!Send` (unlocking a std mutex must happen on the same OS thread that locked it on some platforms, so the stdlib opts it out of `Send` unconditionally). Holding it across `.await` bakes it into the generated state machine, which makes the whole future `!Send`. You'll get exactly the error you saw:

  ```
  error: future cannot be sent between threads safely
  within `impl Future<...>`, the trait `Send` is not implemented for `MutexGuard<'_, i32>`
  ```

  This is a genuinely nice safety net — the compiler is catching the bug for you before it ever runs.

- **If you sidestep that** (e.g. `#[tokio::main(flavor = "current_thread")]`, or you drive the futures directly instead of `tokio::spawn`, so `Send` isn't required): it _will_ compile, and now you get the real runtime deadlock — `t2`'s blocking `.lock()` call parks the one and only OS thread while `t1` is asleep holding the lock. Nothing can ever wake `t1` up to finish its sleep and release the lock. Total, permanent hang.

---

## 4. Fix 1 — use `tokio::sync::Mutex` (what you actually did)

```rust
use std::sync::Arc;
use tokio::sync::Mutex; // async-aware lock: safe to hold across .await
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let ctr = Arc::new(Mutex::new(0));

    let ctr_a = Arc::clone(&ctr);
    let t1 = tokio::spawn(async move {
        let mut guard = ctr_a.lock().await; // async lock — fine to hold across await
        *guard += 1;
        sleep(Duration::from_secs(5)).await; // other tasks just suspend on .lock(), no thread blocked
    });

    let ctr_b = Arc::clone(&ctr);
    let t2 = tokio::spawn(async move {
        let mut guard = ctr_b.lock().await;
        *guard += 1;
    });

    let _ = tokio::join!(t1, t2);
    println!("{}", *ctr.lock().await);
}
```

This compiles and runs fine — `t2` just waits its turn on the async lock while `t1` sleeps; no OS thread is ever blocked. (This costs you: `t1` holds the counter locked for the _entire_ 5-second sleep, so `t2` can't even increment during that window. Correct, but serializes more than necessary — see §6.)

## Fix 2 — keep `std::sync::Mutex`, just don't hold the guard across `.await`

```rust
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let ctr = Arc::new(Mutex::new(0));

    let ctr_a = Arc::clone(&ctr);
    let t1 = tokio::spawn(async move {
        {
            let mut guard = ctr_a.lock().unwrap();
            *guard += 1;
        } // guard dropped HERE, before the await — this is the key fix
        sleep(Duration::from_secs(5)).await;
    });

    let ctr_b = Arc::clone(&ctr);
    let t2 = tokio::spawn(async move {
        let mut guard = ctr_b.lock().unwrap();
        *guard += 1;
    });

    let _ = tokio::join!(t1, t2);
    println!("{}", *ctr.lock().unwrap());
}
```

Same pattern you already used instinctively in your original code — scope the guard so it drops before any `.await`. This is generally the **preferred** fix over Fix 1 when your critical section is short and doesn't itself need to await anything: `std::sync::Mutex` is cheaper (no async machinery) and there's no ambiguity about lock duration.

---

## 5. Recommended final version of your original demo

Same structure as yours, but fixing the practical issue (999 × 5s ≈ 83 min before the program ever exits) and the debug-print:

```rust
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{sleep, Duration};

// Why a Mutex at all?
// 1. Arc<T> only ever hands out `&T` — mutation requires interior mutability.
// 2. #[tokio::main] defaults to a multi-threaded runtime, so spawned tasks can
//    genuinely run in parallel across cores, not just interleave on one thread.
//    `*ctr += 1` is a read-modify-write and needs real synchronization.
//
// Why tokio::sync::Mutex specifically?
// std::sync::Mutex::lock() blocks the OS thread synchronously if contended.
// tokio::sync::Mutex::lock() is a Future: if contended, the task suspends
// (registers a waker) and the executor moves on to other work — no OS thread
// is ever blocked. That's what makes it safe to hold across an .await.

#[tokio::main]
async fn main() {
    let mut tasks = Vec::new();
    let ctr = Arc::new(Mutex::new(0));

    // 4 fast tasks, 1000 increments each, yielding after every increment
    for _ in 0..4 {
        let ctr = Arc::clone(&ctr);
        let t = tokio::spawn(async move {
            for _ in 0..1000 {
                {
                    let mut ctr = ctr.lock().await;
                    *ctr += 1;
                } // guard dropped before yield_now — good practice even with tokio::Mutex
                task::yield_now().await;
            }
        });
        tasks.push(t);
    }

    // 1 slow task. Kept the guard scoped so it's released before the sleep —
    // this is what actually prevents the "hold lock across await" problem,
    // NOT the choice of Mutex type by itself.
    //
    // NOTE: reduced iterations from 999 to 5 and the sleep from 5s to 500ms.
    // 999 x 5s would mean join_all doesn't return for ~83 minutes — that's not
    // a deadlock, it's just a very long, very literal wait.
    {
        let ctr = Arc::clone(&ctr);
        let t = tokio::spawn(async move {
            for _ in 0..5 {
                {
                    let mut ctr = ctr.lock().await;
                    *ctr += 1;
                }
                sleep(Duration::from_millis(500)).await;
            }
        });
        tasks.push(t);
    }

    let _ = join_all(tasks).await;

    // Lock and print the value itself, not the Mutex's Debug wrapper
    println!("final count = {}", *ctr.lock().await);
}
```

Expected output: `final count = 4005` (4 × 1000 + 5).

---

## 6. One extra thing worth knowing (not a bug, just a design note)

In Fix 1 / your original slow task, the lock is held for the _entire_ body between acquiring and the next scope boundary — in your case that's fine since you drop before sleeping. But if you ever find yourself needing to hold a `tokio::Mutex` across a long `.await` (e.g. a slow network call) purely to protect a counter, that's usually a sign to shrink the critical section further, or reach for `AtomicU32`/`AtomicUsize` instead of a `Mutex<u32>` altogether — a bare counter like this doesn't need a lock at all in production code:

```rust
use std::sync::atomic::{AtomicU32, Ordering};
let ctr = Arc::new(AtomicU32::new(0));
ctr.fetch_add(1, Ordering::SeqCst);
```

Worth keeping the `Mutex` version for _this_ exercise since the point is to learn lock semantics — just flagging it so you don't cargo-cult `Mutex<u32>` into real code later.
