## Shared scoreboard

Create `Arc<Mutex<HashMap<String, u32>>>` as a shared scoreboard. Spawn 4 threads — "alice", "bob", "carol", "dave". Each thread increments its own score 5 times (acquiring the lock each time). Main thread prints the final scoreboard after all threads finish. Then add a 5th thread that only reads scores (lock, clone the map, unlock, print) — it must never block the writers longer than necessary.
Cements: locking a complex type, minimizing lock hold time, read vs write access patterns

---

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Shared scoreboard:
    // Arc  -> multiple threads can own/access the same HashMap
    // Mutex -> only one thread can modify/read it at a time
    let mut scores: HashMap<String, u32> = HashMap::new();

    scores.insert(String::from("alice"), 0);
    scores.insert(String::from("bob"), 0);
    scores.insert(String::from("carol"), 0);
    scores.insert(String::from("dave"), 0);

    let scores = Arc::new(Mutex::new(scores));

    let mut handles = Vec::new();

    // Each player increments their own score 5 times.
    // IMPORTANT: lock is acquired for EACH increment, not all 5.
    //
    // lock -> modify -> unlock
    // lock -> modify -> unlock
    //
    // This minimizes how long other threads are blocked.

    for (name, points) in [
        ("alice", 1),
        ("bob", 2),
        ("carol", 3),
        ("dave", 4),
    ] {
        let scores = Arc::clone(&scores);
        let name = String::from(name);

        let handle = thread::spawn(move || {
            for _ in 0..5 {
                let mut scores = scores.lock().unwrap();

                // MutexGuard automatically derefs to HashMap,
                // so we can directly call HashMap::entry().
                let cur_score = scores.entry(name.clone()).or_insert(0);

                // cur_score is &mut u32, so * dereferences it
                // to access the actual u32 value.
                *cur_score += points;
            }
            // MutexGuard is dropped here at the end of each iteration,
            // releasing the lock.
        });

        handles.push(handle);
    }

    // ------------------------------------------------------------
    // Reader thread
    //
    // CRUX:
    // Lock -> clone the HashMap -> unlock -> print.
    //
    // We DON'T print while holding the lock.
    // The reader only blocks writers for the time required to
    // clone the HashMap.
    // ------------------------------------------------------------

    let scores_reader = Arc::clone(&scores);

    let reader = thread::spawn(move || {
        let snapshot = {
            let scores = scores_reader.lock().unwrap();

            // Clone while protected by the Mutex.
            // After this scope ends, the lock is released.
            scores.clone()
        };

        // No lock is held while printing.
        println!("Reader snapshot: {snapshot:?}");
    });

    // Wait for all writer threads.
    for handle in handles {
        handle.join().unwrap();
    }

    // Wait for reader.
    reader.join().unwrap();

    // Main also reads the final scoreboard after all threads finish.
    println!("Final scoreboard: {:?}", scores.lock().unwrap());
}
```
