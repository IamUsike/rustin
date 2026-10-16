# How async works in Rust

Short version: an async task, like a sleeping thread, uses no CPU time while it waits. The difference is who does the waiting. With `std::thread::sleep`, the **kernel** parks a whole **thread**. With `.await`, the **async runtime** (tokio) parks a **task**, and the thread moves on to run other tasks.

## The pieces

| Piece      | What it is                                              | Who provides it              |
| ---------- | ------------------------------------------------------- | ---------------------------- |
| `Future`   | A value that can be polled until it is done             | The language (`std::future`) |
| `async fn` | Compiled into a struct that implements `Future`         | The compiler                 |
| Executor   | Loop that polls the tasks that are ready                | tokio                        |
| Reactor    | Waits on the OS for I/O and timers                      | tokio (epoll on Linux)       |
| `Waker`    | Handle a future leaves behind so it can be polled again | Executor                     |

## 1. A future is a poll function

```rust
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

enum Poll<T> { Ready(T), Pending }
```

Calling an `async fn` runs **nothing**. It only builds the future value. Work happens only when something calls `poll`.

`poll` returns one of two results:

- `Ready(value)`: the future is done.
- `Pending`: the future can't continue yet. Before it returns `Pending`, it must store `cx.waker()` somewhere so it gets polled again later. If it doesn't store the waker, the task never runs again.

## 2. `async fn` becomes a state machine

```rust
async fn fetch() -> usize {
    let a = read_socket().await;
    sleep(Duration::from_secs(1)).await;
    a.len()
}
```

The compiler turns this function into roughly the following enum:

```rust
enum FetchFuture {
    Start,
    WaitingRead(ReadFuture),
    WaitingSleep(SleepFuture, Vec<u8>), // keeps `a` alive across the await
    Done,
}
```

Each `.await` is a point where the function can pause. When polled, `fetch` advances as far as it can. When an inner future returns `Pending`, `fetch` returns `Pending` too. Local variables that are still needed after an `.await` are stored as fields of the enum, not on the stack. Because of this, the future's size is known when the code compiles, and a task needs only one allocation, made when it is spawned.

This is also why `Pin` exists. The state machine can hold references into its own fields, so it must not move in memory once polling starts.

## 3. Walkthrough: `tokio::time::sleep(500ms).await`

1. **First poll:** the executor polls your task, which polls the `Sleep` future.
2. **Register:** `Sleep` adds itself to tokio's timer wheel with a deadline of now + 500ms, stores the waker, and returns `Pending`.
3. **Task yields:** `Pending` goes up through every `.await` to the executor. The task is now parked, and it costs nothing but its memory.
4. **Thread keeps working:** the executor polls other ready tasks on the same thread.
5. **Nothing ready:** the worker thread calls `epoll_wait` with a timeout equal to the next timer deadline. From here it works like `thread::sleep`: the kernel parks the thread, and the core can go idle.
6. **Deadline passes:** `epoll_wait` returns. tokio's driver sees the expired timer and calls `waker.wake()`.
7. **Reschedule:** `wake()` puts the task back on the executor's run queue.
8. **Second poll:** the executor polls the task again. `Sleep` sees the deadline has passed and returns `Ready(())`. Your code continues after `.await`.

Socket I/O follows the same pattern. The socket is registered with epoll. A read that would block returns `Pending`. When epoll reports that data is ready, the waker fires.

## 4. Where the CPU time goes

- **Waiting:** zero CPU time. A parked task is just a struct in memory.
- **Switching:** switching between tasks is a function return plus a function call, with no syscall and no kernel context switch. That's why one thread can handle thousands of tasks.
- **Memory:** an OS thread reserves a stack of about 8MB (virtual). A task takes only the size of its state machine, often a few hundred bytes.

## 5. Blocking is the cardinal sin

The executor regains control only when a future returns `Pending`. Code that never reaches an `.await` holds the thread:

```rust
async fn bad() {
    std::thread::sleep(Duration::from_secs(1)); // kernel parks the *thread*
    // every other task on this worker is stuck for 1s
}
```

Long CPU-bound loops and blocking calls (`std::fs`, sync DB clients, heavy computation) cause the same problem. Fixes:

- Use the async version: `tokio::time::sleep`, `tokio::fs`.
- Move blocking work off the executor with `tokio::task::spawn_blocking(|| ...)`, which runs it on a separate thread pool.

## 6. `.await` vs `spawn` vs `join!`

```rust
a().await; b().await;              // sequential: b starts after a finishes
tokio::join!(a(), b());            // concurrent: same task, both polled, total time ≈ max(a, b)
let h = tokio::spawn(a()); h.await; // separate task: the scheduler can run it on another worker thread
```

Concurrency means many tasks are in progress at once, which `join!` gives you even on one thread. Parallelism means tasks run at the same instant on different cores. Only `spawn` on the multi-threaded runtime gives parallelism.

## 7. Rules of thumb

- Futures are lazy: nothing runs until you `.await` or `spawn` the future.
- `Pending` must come with a stored waker, or the task stalls forever. Library futures handle this for you.
- Code between two `.await` points runs without interruption. Keep it short.
- Async helps I/O-bound work, like many sockets or many waits. It doesn't make CPU-bound work faster.
