## Thread return values

Spawn 4 threads. Each receives a Vec<i32> (different data per thread) and returns its sum. Collect all 4 return values in main using join() and print the total. You must move the vec into each thread — you cannot pass a reference.

> Cements: why threads own their data, JoinHandle::join returns a Result with the return value

```rust
use std::thread;

fn main() {
    // Each Vec is owned by main initially.
    let vecs = [
        vec![1, 2, 3],
        vec![2, 3, 4],
        vec![3, 4, 5],
        vec![4, 5, 6],
    ];

    // Each spawned thread will return an i32 (its sum).
    //
    // Therefore:
    //
    //     thread::spawn(...)
    //
    // gives us:
    //
    //     JoinHandle<i32>
    //
    // We store all four handles in this Vec.
    let mut threads: Vec<thread::JoinHandle<i32>> = Vec::new();

    for v in vecs {
        // `v` is moved out of `vecs` because the loop consumes the array.
        //
        // `move` then moves ownership of `v` into the closure.
        //
        // This is important because the spawned thread may continue
        // executing after this loop iteration has finished.
        //
        // We cannot safely give the spawned thread a reference to `v`.
        let handle = thread::spawn(move || {
            // `v` is now OWNED by this thread.
            //
            // `iter()` borrows the elements of the Vec:
            //
            //     Vec<i32>
            //        |
            //        | iter()
            //        v
            //     Iterator<Item = &i32>
            //
            // `sum()` consumes the iterator, but does NOT consume `v`.
            //
            // The result of `sum()` is an i32.
            let sum: i32 = v.iter().sum();

            // The final expression of the closure is `sum`.
            //
            // Therefore the closure's return type is:
            //
            //     i32
            //
            // Consequently:
            //
            //     thread::spawn(...)
            //
            // returns:
            //
            //     JoinHandle<i32>
            sum
        });

        // Store the JoinHandle so that main can later:
        //
        // 1. Wait for the thread to finish.
        // 2. Retrieve the value returned by the thread.
        threads.push(handle);
    }

    // Accumulator for the results returned by all threads.
    let mut total = 0;

    for handle in threads {
        // `join()` waits for the spawned thread to finish.
        //
        // Because:
        //
        //     handle: JoinHandle<i32>
        //
        // `join()` returns:
        //
        //     Result<i32, Box<dyn Any + Send>>
        //
        // Conceptually:
        //
        //     JoinHandle<T>
        //          |
        //          | join()
        //          v
        //     Result<T, ThreadPanic>
        //
        // Here T = i32.
        //
        // `unwrap()` extracts the i32 from the successful Result.
        let sum: i32 = handle.join().unwrap();

        // Add this thread's returned value to the total.
        total += sum;
    }

    println!("Total: {}", total);
}
```

---

### the core concept

Most imp relationship to understand is

```
Closure return type
        │
        │
        ▼
       i32
        │
        │ thread::spawn(...)
        ▼
JoinHandle<i32>
        │
        │ join()
        ▼
Result<i32, Box<dyn Any + Send>>
        │
        │ unwrap()
        ▼
       i32
```

---

Why No Arc<Mutex<_>>?

Your previous solution used:

```rust
Arc<Mutex<Vec<i32>>>
```

That means threads communicate through shared mutable state:

```
Thread ──┐
Thread ──┼──> Mutex ──> shared Vec
Thread ──┤
Thread ──┘
```

This exercise instead wants return values:

```
Thread 0 ──return──> 6 ──┐
Thread 1 ──return──> 9 ──┤
Thread 2 ──return──> 12 ──┼──> main → 42
Thread 3 ──return──> 15 ──┘
```

So the key distinction is:

```
Arc<Mutex<_>>
→ shared state between threads

JoinHandle<T>
→ thread produces a value T
→ main retrieves it with join()
```
