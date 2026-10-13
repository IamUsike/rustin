# Shared State

## The problem

- After spawning, every connection has its own `HashMap`. A `SET` from one client is invisible to every other client.
- We need one map that all tasks can read and write.

## Two strategies

1. **Guard the state with a mutex.** Every task locks it, does its work, and unlocks.
2. **Give the state to one dedicated task** and let other tasks send it messages (channels). This is covered in a later chapter.

- A mutex is the simple choice for simple data like a `HashMap`.
- A dedicated task is the better choice when the work is async itself (for example, a single shared connection to another server), or when lock contention gets high.

## `Bytes` instead of `Vec<u8>`

- The tutorial switches values to `bytes::Bytes` (add the `bytes` crate).
- `Bytes` is basically `Arc<[u8]>` plus extra features (cheap slicing). `clone()` increments a refcount and does **not** copy the data.
- With `Vec<u8>`, every `GET` does `value.clone()`, which copies the whole value. With `Bytes`, a `GET` of a 1 MB value is just a refcount bump.
- `Frame::Bulk` already takes `Bytes`, so no `.into()` conversion is needed.

## The shared map

```rust
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();              // clone the Arc, not the map
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}
```

```rust
let response = match Command::from_frame(frame).unwrap() {
    Set(cmd) => {
        let mut db = db.lock().unwrap();
        db.insert(cmd.key().to_string(), cmd.value().clone());
        Frame::Simple("OK".to_string())
    }
    Get(cmd) => {
        let db = db.lock().unwrap();
        match db.get(cmd.key()) {
            Some(value) => Frame::Bulk(value.clone()),
            None => Frame::Null,
        }
    }
    cmd => panic!("unimplemented {:?}", cmd),
};
// guard is already dropped here (end of each match arm)
connection.write_frame(&response).await.unwrap();
```

What each layer does:
- `HashMap`: the data.
- `Mutex`: only one task touches the map at a time. It gives mutable access through a shared reference.
- `Arc`: shared ownership across tasks. It satisfies `'static`, because each task owns its own clone of the `Arc`.
- `let db = db.clone()` must come **before** `async move`. Otherwise the first iteration moves the original `db`, and the second iteration does not compile.

## `std::sync::Mutex` vs `tokio::sync::Mutex`

- The tutorial uses `std::sync::Mutex` on purpose. Defaulting to `tokio::sync::Mutex` in all async code is a common mistake.
- `std` mutex: a contended `lock()` blocks the whole worker thread, including every other task on it. This is fine when the critical section is tiny. A `HashMap` insert or get takes nanoseconds.
- `tokio` mutex: a contended `lock().await` yields, so other tasks can run. But every lock costs more (a future, waker registration, a wait queue), and internally it still uses a sync mutex. For short critical sections it is slower and gains nothing.
- The real purpose of the tokio mutex is **holding the lock across an `.await`**, for example a shared IO resource that you must hold while you write to it.
- Rule: lock, do quick work, drop the guard, and only then `.await`.

| Situation | Use |
|---|---|
| Short work, no `.await` while locked | `std::sync::Mutex` (or `parking_lot::Mutex`) |
| Must hold the lock across `.await` | `tokio::sync::Mutex` |
| High contention | Sharding, a concurrent map, or a dedicated task |

## Contention

- Blocking is not the real issue. **Contention** is: many tasks fight for one lock and are serialized behind it.
- With the multi-threaded runtime, N worker threads can all hit the same lock at once.
- Options:
  - **`current_thread` runtime.** Only one thread, so the lock is never contended. All tasks share one core.
  - **Sharding.** Use several mutexes, each guarding part of the keyspace:

    ```rust
    type ShardedDb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

    let mut shard = db[hash(key) % db.len()].lock().unwrap();
    shard.insert(key, value);
    ```

    Two tasks block each other only if their keys land in the same shard. Cost: the shard count is fixed, and an operation that touches keys in several shards (like `MSET` or `KEYS`) needs several locks. Lock them in a fixed order or you risk deadlock.
  - **Concurrent map crates.** `dashmap` (sharded internally), `leapfrog`, `flurry`.
  - **Dedicated task plus channels.** There is no lock at all, and one task owns the map (next chapter).
- Measure before you optimize. For a toy redis, a single `Mutex` is fine.

## Holding a `MutexGuard` across `.await`

```rust
async fn increment_and_do_stuff(mutex: &Mutex<i32>) {
    let mut lock = mutex.lock().unwrap();
    *lock += 1;
    do_something_async().await;   // lock still alive here
}
```

- Spawning this gives `future cannot be sent between threads safely`, because `std::sync::MutexGuard` is `!Send`.
- Even without a spawn (for example on `current_thread`), it is a **deadlock risk**. The task yields while it holds the lock. Another task on the same thread calls `lock()` and blocks the thread. The first task can never resume and unlock.
- `drop(lock)` before the `.await` **does not always satisfy the compiler**. The `Send` check is based on scope, not on `drop` calls. Use a block:

```rust
{
    let mut lock = mutex.lock().unwrap();
    *lock += 1;
}                                  // guard gone
do_something_async().await;
```

- A cleaner fix: wrap the mutex in a struct with **non-async** methods. Then no guard can ever cross an `.await`, because the compiler forbids it:

```rust
struct CanIncrement { mutex: Mutex<i32> }

impl CanIncrement {
    fn increment(&self) {             // sync fn: guard dies before return
        let mut lock = self.mutex.lock().unwrap();
        *lock += 1;
    }
}
```

- Last resort: `tokio::sync::Mutex`, whose guard is `Send`. Check first that you really need the lock across the await.

## Things to keep in mind

- **Mutex poisoning.** A task that panics while it holds a `std` mutex guard poisons the mutex. Every later `lock().unwrap()` panics too. In our server, one bad command could break every future connection, if the panic happens while the guard is alive. Today the `panic!("unimplemented")` happens outside the lock, so the server is safe for now. Options: `lock().unwrap_or_else(|e| e.into_inner())`, or `parking_lot::Mutex`, which has no poisoning.
- **Hidden guard lifetime in `match`.** `match db.lock().unwrap().get(k) { ... }` keeps the guard alive for the **whole** match, because temporaries in the scrutinee live until the end of the match. If a match arm does `.await`, you get the `Send` error again. Bind the result first:

  ```rust
  let value = db.lock().unwrap().get(k).cloned();  // guard dropped at ;
  ```

- **Clone inside the lock, work outside.** Holding the lock while you do slow work (serialization, logging, IO) blocks everyone. Copy out what you need (cheap with `Bytes`), drop the guard, then do the work.
- **`RwLock` is not automatically better.** It helps with read-heavy workloads and long reads. For nanosecond `HashMap` lookups, the extra bookkeeping usually costs more than it saves.
- **`Arc<Mutex<T>>` vs `Mutex<Arc<T>>`.** The first is a shared, mutable `T`. The second is a lock around a pointer, which is useful for swapping in a whole new snapshot. Don't mix them up.
- **The `Db` type alias** keeps signatures readable. Later the tutorial wraps it in a struct, which is a natural place for the non-async-methods pattern above.

## Questions to check yourself

1. Why is `let db = db.clone();` written before `tokio::spawn(async move { .. })` and not inside it? What error do you get if you remove it?
2. Which part of `Arc<Mutex<HashMap<..>>>` satisfies the `'static` bound, and which part allows mutation through a shared reference?
3. Why does the tutorial use `Bytes` and not `Vec<u8>` for values? What does `Bytes::clone` cost compared to `Vec::clone`?
4. In your `process`, where exactly is the `MutexGuard` dropped? Is it alive during `write_frame(..).await`?
5. Explain how holding a `std::sync::MutexGuard` across `.await` can deadlock on a `current_thread` runtime, even though it compiles there.
6. `drop(guard); something.await;` can still fail the `Send` check. Why? What is the reliable fix?
7. When is `tokio::sync::Mutex` the right choice? Give one concrete example from a redis-like server.
8. You shard the db into 16 mutexes. How do you implement `MSET k1 v1 k2 v2` safely if `k1` and `k2` land in different shards?
9. A client sends a command that panics while the db lock is held. What happens to the next client's `SET`? How do you prevent it?
10. Does `match db.lock().unwrap().get(key) { Some(v) => { conn.write_frame(..).await } ... }` compile in a spawned task? Why or why not?
