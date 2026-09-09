## Code Review — async streams / mpsc producer-consumer

### ✅ What's right

- **Channel + `ReceiverStream` combo** — turning an `mpsc::Receiver` into a `Stream` via `ReceiverStream::new(rx)` and driving it with `StreamExt::next()` is the correct, idiomatic way to do this.
- **`publisher` spawning internally** — moving `tx` into `tokio::spawn` inside `publisher` and letting the function return after spawning (not after the loop finishes) is correct. This is _why_ your concurrency actually works.
- **The "why doesn't this deadlock" instinct** — you're right that `main` doesn't hang forever; you're just slightly off on _why_ (see below).
- **`let _ = tx.send(...).await`** — reasonable for a toy example; a real implementation would want to handle the `Err` case, but not a blocker here.

### ⚠️ Bugs / things to fix

**1. Your concurrency reasoning is subtly wrong.**

```rust
//should the publisher and receiver be sequential? No, because once publisher hits
//sleep maybe(or tokio::spawn?), ctrl is given to the runtime and then receiver is
//called.
```

It's not the `sleep` that hands control back — it's `tokio::spawn`. `publisher(tx).await` returns almost instantly because the _body_ of `publisher` has no `.await` point other than creating the spawn (which returns immediately). The spawned task then runs independently on the executor. So by the time `receiver(rx).await` is called, the producer task is already off running on its own — nothing to do with yielding at `sleep`. If you removed the `tokio::spawn` and just looped+sent+slept directly inside `publisher`, `publisher(tx).await` would block until all 7 messages were sent, and _then_ `receiver` would run — proving spawn is what buys you concurrency, not the `sleep`.

**2. Off-by-one on message count.**

```rust
for i in 1..8 {   // sends messages 1..7 — that's 7 messages, not 8
```

If you wanted 8 messages (matching your `//part 1` naming convention or general expectation), use `1..=8` or `1..9`.

**3. Sleep duration doesn't match the stated spec.**

The exercise description says _"a producer sends values every 100ms"_, but the code uses:

```rust
sleep(Duration::from_millis(300)).await;
```

Not wrong, just inconsistent with your own doc comment — worth aligning or updating the comment.

**4. Discarding the `JoinHandle`.**

```rust
tokio::spawn(async move {
    for i in 1..8 { ... }
});
```

You never keep the `JoinHandle`. In this specific program it's harmless because `receiver` only terminates once the channel closes (i.e., once the spawned task drops `tx`), so you implicitly "wait" for it. But if the spawned task **panics**, you'll never know — the panic is silently swallowed unless you're using a panic hook. Better:

```rust
async fn publisher(tx: mpsc::Sender<String>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        for i in 1..=8 {
            if tx.send(format!("message {i}")).await.is_err() {
                break; // receiver dropped, stop producing
            }
            sleep(Duration::from_millis(300)).await;
        }
    })
}
```

and in `main`, optionally `let handle = publisher(tx).await; ... handle.await.unwrap();` if you want to propagate panics/errors explicitly rather than relying on channel-closing as your synchronization signal.

**5. `publisher` doesn't need to be `async` at all.**

Since the only thing it does is call `tokio::spawn` (which is a plain sync function) and return, `async fn publisher(...)` provides no benefit — you're not awaiting anything inside it before returning. This works:

```rust
fn publisher(tx: mpsc::Sender<String>) {
    tokio::spawn(async move {
        for i in 1..=8 {
            let _ = tx.send(format!("message {i}")).await;
            sleep(Duration::from_millis(300)).await;
        }
    });
}
```

Then call it as `publisher(tx);` (no `.await`) in `main`. Marking it `async` when it has no internal `.await` points is a signal to a reviewer that something's off, even if it compiles fine (an `async fn` with no `.await` just resolves instantly, which is exactly what's letting your current code "work" — but it's accidental correctness, not intentional design).

**6. Naming nitpick.**

```rust
let mut resp = ReceiverStream::new(rx);
while let Some(res) = resp.next().await {
```

`resp`/`res` reads like "response," which implies a request/response protocol you don't have here. `stream` / `msg` would be clearer:

```rust
let mut stream = ReceiverStream::new(rx);
while let Some(msg) = stream.next().await {
    println!("{msg}");
}
```

**7. Part 1 (commented out) doesn't fully satisfy the stated exercise.**

Your own spec says: _"filter evens, map to squares, take first 5, **collect**."_ The commented code does filter/map/take but never collects — it just prints in a loop:

```rust
let mut sq = stream.filter(|x| *x % 2 == 0).map(|x| x * x).take(5);
while let Some(v) = sq.next().await {
    println!("{v}");
}
```

If "collect" is a hard requirement (rather than just "consume"), this should be:

```rust
let sq: Vec<_> = tokio_stream::iter(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 16, 20])
    .filter(|x| *x % 2 == 0)
    .map(|x| x * x)
    .take(5)
    .collect()
    .await;

println!("{sq:?}");
```

### Summary

| Issue                                                               | Severity                                    |
| ------------------------------------------------------------------- | ------------------------------------------- |
| Concurrency explanation attributes it to `sleep` instead of `spawn` | Conceptual — worth fixing your mental model |
| `1..8` sends 7, not 8 messages                                      | Minor off-by-one                            |
| 300ms vs documented 100ms                                           | Cosmetic/spec mismatch                      |
| Discarded `JoinHandle` — panics vanish silently                     | Real robustness gap                         |
| `publisher` marked `async` unnecessarily                            | Style/clarity                               |
| `resp`/`res` naming                                                 | Style                                       |
| Part 1 doesn't `collect()` per spec                                 | Functional gap vs. stated exercise          |

Core mechanism (spawn + mpsc + `ReceiverStream`) is solid — the fixes above are mostly about tightening the story between "what the code does" and "what you believe it does," which matters more long-term than the syntax itself.
