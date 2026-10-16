# Channels

## The problem

- We now want a **client** that sends many commands concurrently from different tasks over **one** redis connection.
- `Client::set` / `Client::get` take `&mut self`. Two tasks can't both hold `&mut client`.

```rust
let mut client = client::connect("127.0.0.1:6379").await.unwrap();
let t1 = tokio::spawn(async { client.get("foo").await; });
let t2 = tokio::spawn(async { client.set("foo", "bar".into()).await; }); // won't compile
```

Options that don't work well:
- **One connection per task.** It works, but connections are expensive and you can't reuse them.
- **`std::sync::Mutex<Client>`.** You would have to hold the guard across `.await` (the request is network IO). That gives a `!Send` error and a deadlock risk.
- **`tokio::sync::Mutex<Client>`.** It compiles, but only one request is in flight at a time, and every task waits in line for the lock. It is also a poor fit for pipelining later.

## The fix: message passing to a manager task

- One **manager task** owns the `Client`. Nobody else touches it.
- Other tasks send `Command`s to the manager over an `mpsc` channel.
- Each command carries a `oneshot::Sender` so the manager can send the reply back to the right task.

```
task A ──┐                       ┌── oneshot ──► task A
          ├── mpsc ──► manager ───┤
task B ──┘          (owns Client) └── oneshot ──► task B
```

This is the actor pattern: state lives in one task, and you talk to it with messages. There is no lock, because only one task ever touches the state.

## Tokio channel types

| Channel | Producers | Consumers | Values | Typical use |
|---|---|---|---|---|
| `mpsc` | many | one | many | queue of requests into a manager or worker |
| `oneshot` | one | one | exactly one | reply to a single request |
| `broadcast` | many | many | many, **each receiver sees every value** | events or pub/sub fan-out |
| `watch` | one (or many) | many | only the **latest** value is kept | config or state changes; receivers skip intermediate values |

- Need multi-producer, multi-consumer where each value goes to **one** consumer (a work queue)? Tokio has no such channel. Use the `async-channel` crate (or `flume`).
- Don't use `std::sync::mpsc` in async code. Its `recv()` blocks the worker thread.

## The code

```rust
use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get { key: String, resp: Responder<Option<Bytes>> },
    Set { key: String, val: Bytes, resp: Responder<()> },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);   // ignore: requester may be gone
                }
                Command::Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Get { key: "foo".to_string(), resp: resp_tx })
            .await
            .unwrap();
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(Command::Set { key: "foo".to_string(), val: "bar".into(), resp: resp_tx })
            .await
            .unwrap();
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
```

Points worth noticing:
- `mpsc::channel(32)`: capacity 32. When the buffer is full, `tx.send(..).await` **waits**. That is backpressure.
- `tx.clone()` gives one sender per task. `Sender` is `Clone`, `Receiver` is not.
- `oneshot::Sender::send` is **not async**. It never waits, because there is exactly one slot. It returns `Err(value)` if the receiver was dropped.
- `resp_rx.await` returns `Result<mini_redis::Result<T>, RecvError>`. That is two layers of error: the outer one means "the manager dropped my `resp`", the inner one means "redis or IO failed".
- The manager forwards the **error** to the requester and does not panic. The task that asked decides what to do.

## Backpressure and bounded channels

- Async work is lazy. Nothing happens until something polls it. But channels and `spawn` let work pile up without limit.
- With an **unbounded** queue and a producer faster than the consumer, memory grows until the process dies. There is no error first, only OOM.
- A **bounded** channel makes the producer slow down to the consumer's speed. This is the main reason to prefer `channel(n)` over `unbounded_channel()`.
- The tutorial's advice: always think about bounds. That covers channel capacity, the number of spawned tasks, the number of open connections, and buffer sizes.

## Things to keep in mind

- **The manager stops when all senders are dropped.** `rx.recv()` returns `None` only after every `Sender` is gone. If `main` keeps one `tx` alive and never uses it, `manager.await` hangs forever. The tutorial avoids this by moving `tx` into `t1` and `tx2` into `t2`, so no stray sender stays in `main`.
- **If the manager dies, every caller sees it.** If `client::connect(..).unwrap()` panics, `rx` is dropped. Every `tx.send(..).await` then returns `Err(SendError(cmd))`, and every waiting `resp_rx.await` returns `Err(RecvError)`. Callers must handle both errors and not `unwrap` them blindly.
- **Commands run one at a time.** The manager awaits each `client.get` before it reads the next command. Throughput is at most one command per network round trip. Real clients pipeline: they send many requests and match the replies in order. The actor design makes that possible later without changing callers.
- **Cancelling a request does not cancel the work.** If the requester gives up (timeout, task aborted), it drops `resp_rx`. The manager still runs the command against redis and only fails on `resp.send`. That's why it uses `let _ = resp.send(..)`. A `SET` you "cancelled" may still happen.
- **Ordering.** The channel is FIFO. Messages from one sender arrive in the order sent. Messages from different senders interleave in arrival order, with no ordering between tasks. `t1`'s GET may run before or after `t2`'s SET, so `GOT = None` or `Some("bar")` are both valid output.
- **Enum size.** A `Command` takes the size of its largest variant. Put a huge variant in a `Box` if it bloats every message in the buffer.
- **Choosing the capacity.** The capacity is how many requests can queue before senders wait. Too small stalls producers on every burst. Too large hides an overloaded consumer and uses memory. Start small (32 to 1024) and measure.
- **Mutex vs actor for our server's `db`.** The `HashMap` could also live in a manager task. That trades lock contention for a single serialization point plus channel overhead. For quick, sync operations, a sharded mutex usually wins. An actor shines when the owned thing needs `.await` (a connection or a file).

## Questions to check yourself

1. Why can't you share `client` between two spawned tasks directly? Why is `tokio::sync::Mutex<Client>` still a weak solution?
2. What is the job of the `oneshot` inside each `Command`? Why not send replies back over a second `mpsc`?
3. Why is `oneshot::Sender::send` not async while `mpsc::Sender::send` (bounded) is?
4. What type does `resp_rx.await` return? Describe a situation that produces each of the two error layers.
5. In `main`, you write `let t1 = tokio::spawn(async move { tx.clone().send(..) ..})` and keep the original `tx`. What happens to `manager.await`? Why?
6. The manager's `client::connect` fails and panics. What do `t1` and `t2` see?
7. `t1` sends a `SET` and then gets cancelled by a `timeout` before the reply. Does the key get set? Why does the manager use `let _ =`?
8. The program prints `GOT = Ok(Ok(None))` for the GET. Is that a bug? Explain using channel ordering.
9. Pick: `mpsc`, `oneshot`, `broadcast` or `watch` for (a) a chat room, (b) the current config that many tasks read, (c) a job queue for workers, (d) the reply to one HTTP request. Which one has no tokio channel?
10. Swap `mpsc::channel(32)` for `unbounded_channel()` and let 10,000 tasks send commands in a loop. What happens to memory, and why does the bounded version not have this problem?
