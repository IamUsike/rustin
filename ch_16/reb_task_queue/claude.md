# Code Review — `TaskQueue` Thread Pool

**Note up front:** I don't have a Rust toolchain in this sandbox to actually
compile-check either version, so please `cargo build` / `cargo run` the
corrected code below before trusting it blindly. Everything here is from
tracing the logic by hand.

## What you got right

- You correctly used the `while` loop around `wake.wait(guard)` instead of
  `if` — this is the one thing almost everyone gets wrong with condvars in
  every language, and your comment about spurious wakeups shows you know
  _why_, not just the pattern.
- Dropping the queue lock before running the task (so other workers can pull
  the next item while this one executes) is the right instinct.
- `Arc`-cloning each piece of shared state into the worker closures rather
  than trying to share `&self` across threads is the correct approach given
  `thread::spawn`'s `'static` bound.

## Bugs

### 1. Spurious empty pops (your "dk why" case)

`keep_awake` is a single shared `bool`, not "does the queue have an item
right now." A worker can see `keep_awake == true`, race another worker to
`queue.pop_front()`, lose the race, and get `None` even though nothing is
wrong — it just means someone else already took the only task. This isn't
rare: with 3 workers and 12 tasks it will fire regularly.

The deeper issue: you have **three separate mutexes** (`queue`,
`tasks_pending`, `keep_awake`) all describing one logical fact — "is there
work?" — updated in three separate lock/unlock cycles inside `submit`. They
can never be observed atomically together, so any decision based on one of
them can be stale relative to the others.

### 2. Lost-wakeup race between `submit` and the worker's reset

This is the one worth worrying about most, because unlike bug #1 it doesn't
just print a stray message — it can strand a task in the queue with no
worker ever coming back for it. Trace this interleaving:

1. Worker A finishes the last pending task, locks `tasks_pending`, reads `0`.
2. **Before** Worker A locks `keep_awake` to set it `false`, the main thread
   calls `submit(task_x)`: pushes to the queue, increments `tasks_pending`
   to `1`, locks `keep_awake`, sets it `true`, calls `notify_one()` — but no
   one is asleep yet, so the notification has nothing to wake.
3. Worker A now acquires `keep_awake` and sets it `false`, because it read
   `tasks_pending == 0` _before_ step 2 happened.
4. Result: `task_x` sits in the queue, `keep_awake` is `false`, and every
   worker that's idle goes back to `wake.wait(...)`. Nothing will ever wake
   them for `task_x` unless something else calls `notify_one`/`notify_all`
   later for an unrelated reason.

In your `main()`, `wait_all()` happens to bail you out because it calls
`notify_all()` regardless of `keep_awake`'s value — so the demo works. But
that's incidental: if this pool stayed alive and `submit` were called
sporadically without a `wait_all()` nearby, tasks could sit stuck
indefinitely. A thread pool where correctness depends on the caller
eventually calling shutdown is fragile.

### 3. Busy-spinning during shutdown drain

Once `shutdown` is `true`, your `while` guard (`!*guard && !shutdown`) is
`false` unconditionally, so workers stop respecting `keep_awake` entirely and
just spin: lock → pop → (maybe `None`, print `"dk why"`) → check
`shutdown && queue.is_empty()` → loop. If the queue empties out slightly
before all workers notice, remaining workers spin at 100% CPU rather than
blocking, until the exit check catches up.

### 4. No panic isolation

If a submitted closure panics, it panics inside the worker's `t()` call. The
worker thread itself then dies (unwinds out of `thread::spawn`'s closure).
Two consequences:

- That worker is gone permanently — the pool silently degrades from 3
  workers to 2, then 1, etc., with no replacement.
- `wait_all`'s `task.join().unwrap()` will itself panic, propagating a
  _task's_ bug into your shutdown code and potentially skipping the `.join()`
  of the remaining workers in the `while let Some(task) = self.workers.pop()`
  loop.

### 5. No `Drop` impl

If a `TaskQueue` is dropped without an explicit `wait_all()` call, `shutdown`
is never set. The worker threads are still parked on `wake.wait(...)` inside
their own `thread::spawn`'d stack — they don't get killed by the struct going
out of scope, they just leak forever, blocked on a condvar nobody will ever
notify with the right condition again.

### 6. Minor: task count vs. spec

You asked for "10 print tasks on 3 workers" but `main()` submits 12. Not a
bug, just doesn't match what you described — worth a glance in case it was
a copy-paste artifact from testing.

## The actual fix: collapse three mutexes into one

Every bug above (#1, #2, #3) traces back to tracking "is there work" with
state split across `queue` + `tasks_pending` + `keep_awake`. The standard
producer/consumer pattern uses **one** mutex, and ties the condvar to that
same mutex, so the sleep predicate is checked against the true queue state,
never a proxy for it:

```rust
use std::collections::VecDeque;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

type Task = Box<dyn FnOnce() + Send + 'static>;

/// Everything the wait predicate needs lives behind ONE mutex. The condvar
/// is paired with that same mutex, so "should I sleep?" is always answered
/// from a single consistent snapshot instead of three mutexes that can
/// disagree with each other (this is what fixed your #1 and #2).
struct Shared {
    queue: Mutex<VecDeque<Task>>,
    wake: Condvar,
    shutdown: AtomicBool,
}

pub struct TaskQueue {
    shared: Arc<Shared>,
    workers: Vec<JoinHandle<()>>,
}

impl TaskQueue {
    pub fn new(n: usize) -> Self {
        let shared = Arc::new(Shared {
            queue: Mutex::new(VecDeque::new()),
            wake: Condvar::new(),
            shutdown: AtomicBool::new(false),
        });

        let workers = (0..n)
            .map(|id| {
                let shared = Arc::clone(&shared);
                thread::spawn(move || worker_loop(id, shared))
            })
            .collect();

        Self { shared, workers }
    }

    /// Push a task and wake exactly one sleeper. One new task means at most
    /// one more worker can usefully become busy, so `notify_one` (not
    /// `notify_all`) is both correct and cheaper. Correct because every
    /// push is paired with one wake, and a worker that stays asleep is
    /// asleep precisely because there was nothing left for it when it last
    /// checked the (now-shared) queue.
    pub fn submit<T>(&self, task: T)
    where
        T: FnOnce() + Send + 'static,
    {
        let mut queue = self.shared.queue.lock().unwrap();
        queue.push_back(Box::new(task));
        self.shared.wake.notify_one();
    }

    /// Signal shutdown, wake everyone, drain remaining tasks, join.
    /// Takes `self` by value (not `&mut self`) so the pool can't be
    /// `submit`-ed to after shutdown starts — that race (bug #2) is now a
    /// compile error instead of a runtime possibility.
    pub fn wait_all(mut self) {
        self.shared.shutdown.store(true, Ordering::SeqCst);
        self.shared.wake.notify_all();
        for handle in self.workers.drain(..) {
            // A panicking *task* can't poison this join (see worker_loop),
            // but log rather than unwrap() so one wedged thread can't stop
            // us from joining the rest.
            if let Err(e) = handle.join() {
                eprintln!("worker thread panicked: {e:?}");
            }
        }
    }
}

fn worker_loop(id: usize, shared: Arc<Shared>) {
    loop {
        let mut queue = shared.queue.lock().unwrap();

        // Single predicate, single mutex: block only while there's truly
        // nothing to do AND we haven't been told to stop. There's no
        // "dk why" case here — pop_front() only ever returns None when the
        // loop below has already confirmed shutdown && empty.
        while queue.is_empty() && !shared.shutdown.load(Ordering::SeqCst) {
            queue = shared.wake.wait(queue).unwrap();
        }

        let task = queue.pop_front();
        drop(queue); // release before running the task

        match task {
            Some(t) => {
                // Isolate the task's panic so a bad closure can't kill this
                // worker thread. AssertUnwindSafe is safe to use here
                // specifically because we hold no locks while calling t().
                if panic::catch_unwind(AssertUnwindSafe(t)).is_err() {
                    eprintln!("worker {id}: task panicked");
                }
            }
            None => break, // only reachable when shutdown+empty is true
        }
    }
}

fn main() {
    let pool = TaskQueue::new(3);
    for i in 0..10 {
        pool.submit(move || println!("task {i} running"));
    }
    pool.wait_all();
}
```

### What's structurally different, and why

|                         | Your version                                                                        | Corrected version                                                                            |
| ----------------------- | ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| Shared state            | 3 mutexes (`queue`, `tasks_pending`, `keep_awake`) that must stay in sync manually  | 1 mutex (`queue`); the condvar predicate reads it directly                                   |
| Sleep condition         | proxy flag (`keep_awake`) that can go stale relative to the queue                   | `queue.is_empty()`, always current                                                           |
| Shutdown drain          | busy-spins once `shutdown` is set, because the sleep condition is bypassed entirely | still blocks normally; workers only wake when there's actually a task or actually a shutdown |
| `submit` after shutdown | possible, and can race with the reset (bug #2)                                      | impossible — `wait_all(self)` consumes the pool                                              |
| Task panics             | kills the worker thread; `wait_all` can panic via `.unwrap()`                       | caught per-task; worker keeps running; `wait_all` logs instead of panicking                  |
| Drop without `wait_all` | workers leak, parked forever                                                        | `Drop` impl signals shutdown and joins as a safety net — see below                           |

### Adding `Drop` as a safety net

`wait_all(self)` takes ownership, but `Drop::drop` only ever gets
`&mut self` — it can't move `workers` out of the struct the same way. The
fix is to change `workers` to `Vec<Option<JoinHandle<()>>>` so `drop` can
`.take()` each handle out through a mutable reference, and to factor the
actual shutdown logic into a private method both `wait_all` and `Drop` call:

```rust
pub struct TaskQueue {
    shared: Arc<Shared>,
    // Option so `drop` can .take() each handle out of a `&mut self` — you
    // can't move a JoinHandle out of a Vec<JoinHandle<_>> through a shared
    // reference, but you *can* swap an Option's contents for None.
    workers: Vec<Option<JoinHandle<()>>>,
}

impl TaskQueue {
    pub fn new(n: usize) -> Self {
        let shared = Arc::new(Shared {
            queue: Mutex::new(VecDeque::new()),
            wake: Condvar::new(),
            shutdown: AtomicBool::new(false),
        });

        let workers = (0..n)
            .map(|id| {
                let shared = Arc::clone(&shared);
                Some(thread::spawn(move || worker_loop(id, shared)))
            })
            .collect();

        Self { shared, workers }
    }

    pub fn submit<T>(&self, task: T)
    where
        T: FnOnce() + Send + 'static,
    {
        let mut queue = self.shared.queue.lock().unwrap();
        queue.push_back(Box::new(task));
        self.shared.wake.notify_one();
    }

    /// Explicit, blocking shutdown — still the preferred way to end a pool,
    /// since it lets you observe join errors. `Drop` below exists as a
    /// safety net for the case where someone forgets to call this.
    pub fn wait_all(mut self) {
        self.shutdown_and_join();
    }

    /// Shared logic between `wait_all` and `Drop::drop`.
    fn shutdown_and_join(&mut self) {
        self.shared.shutdown.store(true, Ordering::SeqCst);
        self.shared.wake.notify_all();
        for handle in self.workers.iter_mut().filter_map(Option::take) {
            if let Err(e) = handle.join() {
                eprintln!("worker thread panicked: {e:?}");
            }
        }
    }
}

impl Drop for TaskQueue {
    fn drop(&mut self) {
        // Runs if the pool is dropped without an explicit wait_all() —
        // e.g. it goes out of scope, or an earlier `?` returns early.
        // Without this, worker threads stay parked on wake.wait(...)
        // forever: nothing else will ever set `shutdown` or notify them.
        self.shutdown_and_join();
    }
}
```

Notes on this shape:

- **`wait_all` and `Drop` share one path** (`shutdown_and_join`), so there's
  only one place that can get the shutdown logic wrong. Calling
  `wait_all()` explicitly and then letting the value drop is harmless —
  `workers` is already empty by the time `drop` runs, so its loop does
  nothing.
- **`filter_map(Option::take)` instead of `.pop()`**: the earlier version's
  `wait_all` used `while let Some(task) = self.workers.pop()`, joining in
  reverse order. Order doesn't matter for correctness (each thread finishes
  independently), but this reads as "drain everything in place" and works
  from `&mut self` rather than needing ownership.
- **Never `.unwrap()` or `panic!` inside `Drop::drop`.** If a `drop` panics
  while the program is already unwinding from a panic elsewhere, Rust
  aborts the whole process instead of just failing the one thread — hence
  logging join errors rather than unwrapping them, same as before.
- **This is a safety net, not the primary API.** `wait_all()` is still how
  you should normally shut the pool down, since you get to decide what "a
  worker panicked" means for your program. `Drop` firing is the "someone
  forgot" case — silent-but-safe cleanup, not a substitute for calling it
  yourself.

### On `tasks_pending`

You weren't wrong to want a pending-count — it's just doing a job that
overlaps with what the queue mutex already tells you here. If you later want
a pool that can **pause and drain a batch without permanently shutting
down** (e.g. a "wait for currently queued work, then keep accepting more"
barrier, as opposed to "shut down forever"), _that's_ a real use for a
separate counter + condvar — but it's a distinct feature from `wait_all`
as currently specified, worth designing on purpose rather than folding into
the same three variables that caused the races above.
