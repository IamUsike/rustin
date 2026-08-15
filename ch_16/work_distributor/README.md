## Work distributor

You have a list of 12 "jobs" (just integers 1–12). Spawn 3 worker threads. Distribute jobs across workers using a channel — main sends all 12 jobs down one tx, all 3 workers share cloned receivers... wait, you can't clone an mpsc Receiver. Figure out why, then solve it a different way: send jobs round-robin across 3 separate channels (one per worker). Collect results from a shared result channel.

> Cements: why Receiver isn't Clone (it would break single-consumer guarantee), round-robin distribution

---

### 1. Intended Solution

- each worker gets its own channel

```
                    ┌── tx0 ──► Worker 0
                    │
Main ───────────────┼── tx1 ──► Worker 1
                    │
                    └── tx2 ──► Worker 2

Worker 0 ──┐
Worker 1 ──┼──► shared result channel ──► Main
Worker 2 ──┘
```

```rust
use std::{
    sync::mpsc,
    thread,
};

fn main() {
    // ------------------------------------------------------------
    // Three separate channels:
    //
    // tx0 -> rx0 -> Worker 0
    // tx1 -> rx1 -> Worker 1
    // tx2 -> rx2 -> Worker 2
    // ------------------------------------------------------------
    let (tx0, rx0) = mpsc::channel();
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    // Store the three SENDERS so main can distribute jobs.
    //
    // This is an array, not a Vec:
    // [Sender<T>; 3]
    //
    // We know there will always be exactly 3 workers.
    let worker_senders = [tx0, tx1, tx2];

    // ------------------------------------------------------------
    // Shared result channel.
    //
    // All workers will send their results through this channel.
    //
    // txr can be cloned because Sender implements Clone.
    // rxr remains the single receiver.
    // ------------------------------------------------------------
    let (txr, rxr) = mpsc::channel();

    // ------------------------------------------------------------
    // Spawn workers
    // ------------------------------------------------------------

    let txr0 = txr.clone();

    let worker0 = thread::spawn(move || {
        for num in rx0 {
            // Worker 0: multiply by 0
            let result = num * 0;

            txr0.send(result).unwrap();
        }

        // rx0 ended because its Sender (tx0) was dropped.
        // Therefore this worker finishes.
    });

    let txr1 = txr.clone();

    let worker1 = thread::spawn(move || {
        for num in rx1 {
            // Worker 1: multiply by 1
            let result = num * 1;

            txr1.send(result).unwrap();
        }
    });

    // Move the original txr into worker 2.
    //
    // We don't need to clone it again.
    let worker2 = thread::spawn(move || {
        for num in rx2 {
            // Worker 2: multiply by 2
            let result = num * 2;

            txr.send(result).unwrap();
        }
    });

    // Store worker handles so we can join them later.
    let workers = vec![worker0, worker1, worker2];

    // ------------------------------------------------------------
    // ROUND-ROBIN DISTRIBUTION
    // ------------------------------------------------------------
    //
    // Jobs:
    //
    // 1 -> Worker 0
    // 2 -> Worker 1
    // 3 -> Worker 2
    // 4 -> Worker 0
    // 5 -> Worker 1
    // 6 -> Worker 2
    // ...
    //
    // i % 3 determines which worker gets the job.
    //
    // IMPORTANT:
    // This guarantees JOB DISTRIBUTION order.
    // It does NOT guarantee RESULT order because workers
    // execute concurrently.
    //
    // worker_senders[i % 3]
    //
    // i = 1 -> 1 % 3 = 1 -> Worker 1
    // i = 2 -> 2 % 3 = 2 -> Worker 2
    // i = 3 -> 3 % 3 = 0 -> Worker 0
    //
    // If you want the first job to go to Worker 0, using
    // zero-based jobs makes the mapping visually simpler.
    // Here we deliberately use jobs 1..=12 as requested.
    // ------------------------------------------------------------

    for i in 1..=12 {
        worker_senders[(i - 1) % 3]
            .send(i)
            .unwrap();
    }

    // ------------------------------------------------------------
    // IMPORTANT: close all worker input channels.
    //
    // worker_senders owns:
    //
    // tx0, tx1, tx2
    //
    // Once these are dropped, the corresponding receivers know:
    //
    // "No more jobs will ever arrive."
    //
    // Therefore:
    //
    // rx0 iteration ends
    // rx1 iteration ends
    // rx2 iteration ends
    // ------------------------------------------------------------
    drop(worker_senders);

    // ------------------------------------------------------------
    // Wait for all workers to finish.
    //
    // Once workers finish, their result Senders are dropped:
    //
    // txr0 -> dropped
    // txr1 -> dropped
    // txr  -> dropped
    //
    // At that point rxr becomes disconnected.
    // ------------------------------------------------------------

    for worker in workers {
        worker.join().unwrap();
    }

    // ------------------------------------------------------------
    // Collect results.
    //
    // Because all workers have finished, ALL result Senders
    // have now been dropped.
    //
    // Therefore this loop eventually terminates.
    // ------------------------------------------------------------

    let mut results = Vec::new();

    for result in rxr {
        results.push(result);
    }

    println!("Final results: {results:?}");
}
```

---

### 2. Alternative - one Receiver shared with `Arc<Mutex<_>>`

- instead of

```
3 channels
   ↓
3 receivers
   ↓
3 workers
```

- we use

```
             ┌── Worker 0
             │
jobs ──► Receiver
             │
             ├── Worker 1
             │
             └── Worker 2
```

- every worker gets a clone of arc

```
                    Arc
                     │
                   Mutex
                     │
                  Receiver
                 /    |    \
               W0    W1    W2
```

- The Mutex ensures that two workers don't simultaneously call recv().

```rust
use std::{
    sync::{
        mpsc,
        Arc,
        Mutex,
    },
    thread,
};

fn main() {
    // ------------------------------------------------------------
    // ONE job channel.
    //
    // Unlike the previous solution, we only have ONE Receiver.
    // ------------------------------------------------------------
    let (tx, rx) = mpsc::channel();

    // Receiver cannot be cloned.
    //
    // So instead we put it inside:
    //
    // Arc<Mutex<Receiver<T>>>
    //
    // Arc = allow multiple threads to own/access the same object.
    //
    // Mutex = only one worker can access the Receiver at a time.
    let rx = Arc::new(Mutex::new(rx));

    // Shared result channel.
    let (txr, rxr) = mpsc::channel();

    let mut workers = Vec::new();

    // ------------------------------------------------------------
    // Spawn 3 workers.
    // ------------------------------------------------------------

    for worker_id in 0..3 {
        // Clone the Arc, NOT the Receiver.
        //
        // All workers point to the same Receiver.
        let rx = Arc::clone(&rx);

        // Each worker needs its own Sender clone.
        let txr = txr.clone();

        let worker = thread::spawn(move || {
            loop {
                // ------------------------------------------------
                // Lock the Receiver.
                //
                // Only one worker can be inside this section at
                // a time.
                // ------------------------------------------------

                let result = {
                    let receiver = rx.lock().unwrap();

                    // recv() waits for a job.
                    //
                    // We only hold the lock while receiving.
                    receiver.recv()
                };

                // ------------------------------------------------
                // IMPORTANT:
                //
                // The MutexGuard is dropped here.
                //
                // Therefore another worker can now acquire
                // the Mutex and receive the next job.
                // ------------------------------------------------

                let num = match result {
                    Ok(num) => num,

                    // Channel closed -> no more jobs.
                    Err(_) => break,
                };

                // ------------------------------------------------
                // Process the job OUTSIDE the Mutex.
                //
                // This is important.
                //
                // We don't want Worker 0 processing a huge job
                // while holding the Receiver lock and preventing
                // Workers 1 and 2 from receiving jobs.
                // ------------------------------------------------

                let result = num * worker_id;

                // Send result through the shared result channel.
                txr.send(result).unwrap();
            }
        });

        workers.push(worker);
    }

    // ------------------------------------------------------------
    // Send all jobs through ONE channel.
    // ------------------------------------------------------------

    for i in 1..=12 {
        tx.send(i).unwrap();
    }

    // ------------------------------------------------------------
    // Close the job channel.
    //
    // Once tx is dropped:
    //
    // recv()
    //    ↓
    // eventually returns Err
    //    ↓
    // worker breaks out of loop
    //    ↓
    // worker finishes
    // ------------------------------------------------------------

    drop(tx);

    // We no longer need the original result Sender in main.
    // Workers each have their own clone.
    drop(txr);

    // ------------------------------------------------------------
    // Wait for workers.
    // ------------------------------------------------------------

    for worker in workers {
        worker.join().unwrap();
    }

    // ------------------------------------------------------------
    // All worker result Senders are now dropped.
    // Therefore rxr is closed and this loop terminates.
    // ------------------------------------------------------------

    let mut results = Vec::new();

    for result in rxr {
        results.push(result);
    }

    println!("Final results: {results:?}");
}
```
