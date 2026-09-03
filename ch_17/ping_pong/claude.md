# Code Review: Async Ping-Pong with `tokio::sync::mpsc`

**Verdict:** Solid — the shutdown/drop reasoning is genuinely the hard part of this exercise and you got it right. A few design/style points below, roughly in order of importance.

---

## ✅ What you got right

### 1. The drop-chain shutdown logic

Your end-of-file comment nails the core insight of this exercise:

```
txa moves into `loo` (async move) → loo finishes its 5 sends → txa drops
  → rxa.recv() returns None → task_b's while-loop exits → task_b's async block
  returns → txb (local to that block) drops → rxb.recv() returns None
  → lis_a exits → task_a completes
```

This is _the_ thing people get wrong in this exercise (forgetting `move` and ending up with a sender that never drops, so the program hangs forever waiting on a `recv()` that will never return `None`). You correctly identified why `move` on the `loo` block is required, and your comment explaining it is accurate. Nice.

One nuance worth internalizing though: it's not that "rxa has to drop" _causes_ txb to drop — `rxa` and `txb` are just two local bindings in `task_b`'s async block. They both drop together (RAII) once that block _returns_, and it returns because `rxa.recv()` observed a closed channel (all senders, i.e. `txa`, gone). So the trigger is "the channel closes," not "a specific variable drops" — the variable drop is a downstream _consequence_, not the mechanism.

### 2. Correct use of `join!` to run two futures on one task

Running `lis_a` and `loo` concurrently inside `task_a` via `tokio::join!` (rather than spawning a third task) is the right call — they share state-ish concerns (both are "task A's" logic) and don't need their own OS-scheduled task.

### 3. Understanding _why_ `yield_now()` is needed

This is the subtlest correct insight in your code. `Sender::send().await` on a channel with free capacity resolves immediately (`Poll::Ready` on first poll) — it doesn't actually suspend and hand control back to the executor. So without `yield_now()`, `loo` could in principle blast through all 5 sends without `lis_a` ever getting polled in between (in practice `join!` polls both futures each time either wakes, but relying on that instead of being explicit is fragile). Explicitly yielding after each send is the correct fix and shows you understand the difference between "an await point" and "an await point that actually yields."

---

## 🔧 Worth reconsidering

### 1. This is _pipelined_ ping-pong, not _turn-based_ ping-pong

This is the main functional/design point. Your `loo` fires all 5 pings without waiting for the corresponding pong:

```rust
let loo = async move {
    for _ in 0..5 {
        txa.send(String::from("ping")).await.unwrap();
        yield_now().await;
    }
};
```

Meanwhile pongs are drained concurrently by `lis_a`. With buffer size 32 this works fine and you'll get 5 pings and 5 pongs total — but it's not actually "ping, wait for pong, ping, wait for pong," it's "fire 5 pings, react to pongs whenever they show up." Whether that matters depends on what the exercise wants, but if the intent is a literal back-and-forth handshake (which "ping-pong" usually implies), the loop should wait for its own pong before sending the next ping:

```rust
let ping_pong = async move {
    for _ in 0..5 {
        txa.send(String::from("ping")).await.unwrap();
        // wait for THIS round's pong before continuing
        if let Some(msg) = rxb.recv().await {
            println!("{msg} received");
        }
    }
    // txa and rxb both drop here, closing both channels
};
tokio::join!(ping_pong); // no separate listener needed
```

This also lets you delete the separate `lis_a` future entirely — turn-based ping-pong doesn't need two concurrently-polled halves in task A, since A is never doing two things at once by definition.

### 2. Spawned task panics/errors are silently discarded

```rust
let _ = tokio::join!(task_a, task_b);
```

`tokio::spawn` returns a `JoinHandle<T>`, and awaiting it gives `Result<T, JoinError>` (panics inside the task surface here). Binding straight to `_` throws that away — if `task_b` panicked on the `unwrap()` below, you'd get no indication at the join site. Prefer:

```rust
let (a, b) = tokio::join!(task_a, task_b);
a.unwrap();
b.unwrap();
```

so a panicked task actually fails loudly instead of vanishing.

### 3. Naming

`loo` and `lis_a` read as noise once you're not the one who just wrote them (`loo` in particular reads like a typo for "loop" — which, per your comment, it deliberately avoids as a keyword, but a name like `ping_sender` costs nothing and avoids the ambiguity). Minor, but reviewers/future-you will thank you.

### 4. `.unwrap()` on `send()`

Fine for an exercise, but worth a mental note: this panics if the receiver is dropped first. In this specific program that can't happen before all sends complete (by your own drop-order reasoning), so it's safe here — just flagging it as the kind of thing that bites you the moment the shutdown order changes.

---

## Summary

| Aspect                                 | Status                                                             |
| -------------------------------------- | ------------------------------------------------------------------ |
| Channel closing / shutdown reasoning   | ✅ Correct, well explained                                         |
| `move` semantics                       | ✅ Correct                                                         |
| Understanding of `yield_now` necessity | ✅ Correct and non-obvious                                         |
| Ping/pong turn-taking semantics        | ⚠️ Pipelined, not strict alternation — may or may not match intent |
| Task error propagation                 | ⚠️ Swallowed via `let _ =`                                         |
| Naming                                 | 🧹 Minor cleanup                                                   |
