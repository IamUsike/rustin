# Spawning

## The problem with the first version

```rust
loop {
    let (socket, _) = listener.accept().await.unwrap();
    process(socket).await;
}
```

- Inside one async fn, code is sequential. `.await` means "this task does not continue until this future is done".
- So the server handles one client at a time. Client B connects while A is being processed: the kernel queues B in the listen backlog, and B waits until `process(A)` returns.
- The thread is not blocked during the wait (the task is parked and the runtime sleeps in `epoll_wait`). But no other task exists, so nothing else can use that free time.
- `async` alone gives you no concurrency. You get it only from more than one task (`tokio::spawn`) or from combinators inside one task (`join!`, `select!`).

## The fix: one task per connection

```rust
loop {
    let (socket, _) = listener.accept().await.unwrap();
    tokio::spawn(async move {
        process(socket).await;
    });
}
```

- `tokio::spawn` gives the future to the runtime as a new task and returns at once. The loop goes straight back to `accept`.
- It returns a `JoinHandle<T>`. `handle.await` gives `Result<T, JoinError>`.
- Dropping the `JoinHandle` (like above) does **not** cancel the task. The task becomes detached and keeps running.

## Tasks vs threads

- A task is a green-thread-like unit that the tokio scheduler manages, not the OS.
- Tasks are cheap: one allocation, about 64 bytes plus the size of the future's state. Thousands or millions of tasks are fine. Thousands of OS threads are not.
- Tasks are **stackless**. The compiler turns an `async fn` into a state machine, and locals that live across an `.await` are stored inside the future. Big locals held across an `.await` make the task bigger.
- Scheduling is **cooperative**. A task gives up the thread only at an `.await` that is not ready yet. A CPU-heavy loop or a blocking call (`std::thread::sleep`, sync file IO, heavy hashing) starves the other tasks on that worker thread. Use `tokio::task::spawn_blocking` for those, or the async versions (`tokio::time::sleep`).
- `#[tokio::main]` starts the multi-threaded runtime by default: one worker thread per core with work stealing. A spawned task can move between threads at any `.await`. This is why the `Send` bound exists (see below).
- Concurrency vs parallelism: tasks are concurrent (they interleave). They are also parallel only if the runtime has more than one worker thread. `#[tokio::main(flavor = "current_thread")]` gives concurrency on one thread.

## The `'static` bound

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
```

- The task can outlive the function that spawned it, so it must not borrow anything from that function.
- This does not compile:

```rust
let v = vec![1, 2, 3];
tokio::spawn(async {
    println!("{:?}", v); // borrows v
});
```

- Fix it with `async move` so that the task owns `v`.
- Nuance: `T: 'static` does **not** mean "lives forever". It means "the type holds no non-static references". An owned `String` is `'static` and can be dropped whenever you like.
- Need the same data in many tasks? Use `Arc<T>` and clone the `Arc` into each task. `Arc::clone` only increments a counter.
- Common mistake: `async move` inside a loop moves the variable on the first iteration, so the second iteration errors. Clone before the spawn:

```rust
let db = db.clone();          // clone of the Arc, not the data
tokio::spawn(async move { /* uses db */ });
```

## The `Send` bound

- The task can move to another thread at any `.await`. So everything stored in the future's state, meaning every value that lives **across** an `.await`, must be `Send`.
- Values that are created and dropped between two `.await` points do not matter.

```rust
// does NOT compile: rc lives across the .await
tokio::spawn(async {
    let rc = Rc::new("hello");
    yield_now().await;
    println!("{}", rc);
});

// compiles: rc dropped before the .await
tokio::spawn(async {
    {
        let rc = Rc::new("hello");
        println!("{}", rc);
    }
    yield_now().await;
});
```

- Error to look for: `future cannot be sent between threads safely`. The compiler points at the `.await` where the non-`Send` value is still alive.
- Usual culprits: `Rc`, `RefCell` borrows, and **`std::sync::MutexGuard` held across an `.await`**. That last one also risks deadlock. Lock, do the work, drop the guard, and only then `.await`.
- Note: an explicit `drop(x)` does not always satisfy the compiler. A scope block `{ ... }` is the reliable fix.

## Storing values: the bug in the current code

```rust
async fn process(socket: TcpStream) {
    let mut db = HashMap::new();
    ...
}
```

- `db` is created **per connection**. Client A runs `SET hello world`. Client B runs `GET hello` and gets `nil`. Even the same client loses everything after it reconnects.
- Each task has its own private `HashMap`. Nothing is shared.
- Fix (next chapter, "Shared state"): create one `Arc<Mutex<HashMap<..>>>` in `main` and clone the `Arc` into each task.

## Things to keep in mind

- **Panics stay inside one task.** `unwrap()` or `panic!("unimplemented")` inside `process` kills only that task. The runtime catches the panic, prints it, and the server keeps running. That client's connection drops. If you `.await` the `JoinHandle`, you get `Err(JoinError)` where `is_panic()` returns true.
- **When `main` returns, the runtime shuts down** and all spawned tasks are dropped mid-work. No "wait for detached tasks" behaviour. Keep the handles and `.await` them, or use a `JoinSet`, if the work must finish.
- **Unbounded spawning.** One task per connection with no limit means a flood of connections creates a flood of tasks and memory use. Real servers cap this, for example with a `tokio::sync::Semaphore`.
- **`accept().await.unwrap()` kills the whole server** on errors such as "too many open files" (EMFILE). That unwrap runs in `main`, not in a spawned task. Real servers log the error and continue the loop.
- **`read_frame().await.unwrap()`** returns `Err` when a client disconnects in the middle of a frame. Today that panics the task. A clean disconnect returns `Ok(None)` and ends the `while let` loop.
- **Order is not guaranteed.** Two spawned tasks can run in any order and interleave at every `.await`.

## Questions to check yourself

1. The server uses `process(socket).await` with no spawn. Client A connects but sends nothing. What happens to client B? Why does the thread itself stay idle and not blocked?
2. Remove `move` from `async move { process(socket).await }`. What error do you get, and why does `'static` need it?
3. Is `String` `'static`? Is `&String`? Is `&'static str`? Explain with the "holds no non-static references" definition.
4. Does this compile in a spawned task? Why or why not?
   ```rust
   let m = std::sync::Mutex::new(0);
   let guard = m.lock().unwrap();
   tokio::time::sleep(Duration::from_millis(1)).await;
   println!("{}", *guard);
   ```
5. Client A runs `SET foo 1` and then client B runs `GET foo`. What does B get with the current code? What is the minimal change that makes B see `1`?
6. You spawn a task and immediately drop its `JoinHandle`. Is the task cancelled? What happens to it when `main` returns?
7. A client sends an unsupported command (`PING`). Trace what happens: which task panics, what does the client see, and does the server stay up?
8. A spawned task runs `loop { counter += 1; }` with no `.await`. What happens to the other tasks with `current_thread`? With the multi-threaded runtime?
9. Why can a tokio task move between threads, while a normal function call cannot be paused halfway and continued on another thread?
10. Why does `tokio::spawn` need `F::Output: Send + 'static` and not only `F: Send + 'static`?
