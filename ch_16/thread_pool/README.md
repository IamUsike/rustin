# Mini thread pool from scratch

Build a ThreadPool struct with new(n: usize) that spawns n worker threads, and execute(f: impl FnOnce() + Send + 'static) that sends a job to an available worker. Workers sit in a loop waiting for jobs via a shared channel. Use Arc<Mutex<Receiver>> so all workers share the same job queue (this is the one valid case for Mutex-wrapped Receiver). Implement Drop for ThreadPool that sends a shutdown signal and joins all threads. Test with 10 jobs on 3 workers.

> Cements: everything — spawn, move, Arc, Mutex, channels, FnOnce as a trait object, graceful shutdown

---

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

// Type alias for a boxed, type-erased job.
// Same as your original — this part was already correct.
// `dyn FnOnce() + Send + 'static` because:
//   - FnOnce: a job runs exactly once, then is consumed
//   - Send: the box crosses a thread boundary (created on main thread, run on worker)
//   - 'static: no borrowed data — the closure must own everything it touches,
//     since we don't know when a worker will actually execute it
type Task = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<JoinHandle<()>>,

    // CHANGE: dropped the `receiver: Arc<Mutex<Receiver<Task>>>` field you had.
    // Reasoning: after `new()` spawns the workers, each worker thread already
    // holds its own `Arc::clone`. The pool struct itself never calls
    // `.recv()` or otherwise touches the receiver again — keeping it around
    // was a dead field that did nothing but hold a refcount open.
    // (If you *wanted* to inspect the queue from the pool later, e.g. to
    // check pending job count, you'd have a reason to keep it. Absent that,
    // drop it — every field should earn its place.)
    sender: Option<mpsc::Sender<Task>>,
}

impl ThreadPool {
    fn new(n: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Task>();

        // Still correct in your version: Mutex is needed here because
        // mpsc::Receiver is not Sync — multiple threads can't call .recv()
        // on it concurrently without external synchronization. Wrapping it
        // in Arc<Mutex<_>> is the textbook justified use of Mutex-around-a-
        // Receiver (as opposed to using a Mutex to protect ordinary data).
        let receiver = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(n);
        for id in 0..n {
            // Each worker gets its own strong reference to the shared queue.
            let receiver = Arc::clone(&receiver);

            workers.push(thread::spawn(move || loop {
                // IMPORTANT (this was already right in your code, worth
                // restating): lock, recv, and let the MutexGuard drop
                // *before* running the job. Because `.lock().unwrap().recv()`
                // is a temporary, the guard is released as soon as `recv()`
                // returns — so the job body runs without holding the lock.
                // If you did `let guard = receiver.lock().unwrap(); let job = guard.recv();`
                // and ran the job before `guard` goes out of scope, only one
                // worker could ever be busy at a time — you'd have serialized
                // your "pool" back into a single thread.
                match receiver.lock().unwrap().recv() {
                    Ok(job) => {
                        job(); // consumes the FnOnce
                        println!("worker {id} completed some task");
                    }
                    // CHANGE: your version already handled this correctly
                    // (Err(_) => break), keeping it — recv() returns Err
                    // only once every Sender is dropped, i.e. the channel
                    // is permanently closed. That's our shutdown signal.
                    Err(_) => break,
                }
            }));
        }

        Self { workers, sender: Some(tx) }
    }

    // CHANGE: signature. Your original was:
    //     fn execute(&self, f: Task)
    // which forces the *caller* to box the closure before calling execute,
    // and doesn't match the spec (`impl FnOnce() + Send + 'static`).
    // It also caused the bug in your `main`: since `f` was already a
    // `Box<dyn FnOnce()...>`, doing `Box::new(f)` boxes the box (an extra
    // needless allocation — a `Box<Box<dyn FnOnce()>>` that then gets
    // coerced back down), and reusing the same `f` across loop iterations
    // doesn't work because Box<dyn FnOnce()> isn't Copy — it's moved into
    // execute() on the first call, gone on the second.
    //
    // Making execute generic over F puts the boxing inside the pool, so
    // callers just hand over an ordinary closure each time, like a real
    // thread::spawn-style API.
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Task = Box::new(f);
        // .as_ref() instead of .clone() on the Option<Sender>:
        // we only need a borrow to call .send(), no need to clone the
        // sender itself (Sender is cheaply cloneable, but there's no
        // reason to do it per-call — we're not moving it anywhere).
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Same idea as your original, kept as-is — this is the correct
        // shutdown mechanism:
        // 1. `.take()` replaces `self.sender` with `None`, giving us
        //    ownership of the actual Sender so we can drop it explicitly.
        // 2. Dropping the *last* Sender closes the channel — every
        //    worker's blocked `.recv()` call then wakes up with `Err(_)`,
        //    which is how they know to `break` out of their loop.
        //    (You need this: if you just dropped the ThreadPool without
        //    ever closing the channel, workers would block on `.recv()`
        //    forever and `.join()` would hang.)
        drop(self.sender.take());

        // 3. Join every worker so `drop()` doesn't return until all
        //    in-flight jobs have actually finished running.
        //    `.pop()` avoids the borrow-checker issue you'd hit trying to
        //    `for worker in self.workers` (that would try to move out of
        //    a `&mut self.workers`, which isn't allowed) — popping owns
        //    each JoinHandle one at a time instead.
        while let Some(worker) = self.workers.pop() {
            worker.join().unwrap();
        }
    }
}

fn main() {
    let tp = ThreadPool::new(3);

    // CHANGE: this is the actual bug fix from your original.
    // Your version built ONE closure `f` outside the loop, then tried to
    // `Box::new(f)` it fresh each iteration — but `f` itself gets moved
    // into that Box on the first iteration and is unavailable afterward.
    // That's a compile error, not a runtime one: the borrow checker
    // rejects `main` before it ever runs.
    //
    // Fix: construct a *new* closure inside the loop body, one per
    // iteration. Each `move || {...}` captures its own copy of `i`
    // (i32 is Copy) and owns everything it needs — no shared closure
    // reused across iterations.
    for i in 0..10 {
        tp.execute(move || {
            thread::sleep(Duration::from_millis(500));
            println!("job {i} done");
        });
    }

    // No explicit join here — `tp` goes out of scope at the end of `main`,
    // which runs `Drop::drop`, which closes the channel and joins all
    // three worker threads. That's what makes shutdown "graceful": by the
    // time `main` returns, all 10 jobs have definitely completed.
}
```
