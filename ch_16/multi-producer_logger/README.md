## Multi-producer logger

Spawn 3 "worker" threads. Each worker does some fake work (sleep random ms) then sends log messages like "worker 2: task A done" down a shared channel. A single "logger" thread receives ALL messages and prints them with a timestamp prefix. Main waits for all workers to finish, then drops the sender so the logger thread exits cleanly.

> Cements: tx.clone() for multiple producers, channel closes when all senders drop, logger pattern

---

```rust
use rand::RngExt;
use std::{
    sync::mpsc,
    thread,
    time::Duration,
};

fn main() {
    // ============================================================
    // 1. CREATE THE CHANNEL
    // ============================================================
    //
    // tx = Sender
    // rx = Receiver
    //
    // mpsc = multiple producer, single consumer.
    //
    // We will have:
    //
    //     Worker 0 ──┐
    //     Worker 1 ──┼──> tx/channel ──> rx ──> Logger
    //     Worker 2 ──┘
    //
    // Each worker gets its own clone of tx.
    // The logger gets the single receiver.
    //

    let (tx, rx) = mpsc::channel();

    let mut handles = Vec::new();

    // ============================================================
    // 2. SPAWN THE WORKERS
    // ============================================================

    for i in 0..3 {
        // IMPORTANT:
        //
        // We clone BEFORE spawning.
        //
        // `thread::spawn(move || ...)` moves everything captured
        // by the closure into the new thread.
        //
        // If we moved the original `tx`, the main thread couldn't
        // keep its Sender.
        //
        // So each worker gets its own Sender clone.
        let tx1 = tx.clone();

        let handle = thread::spawn(move || {
            // Each worker has its own random-number generator.
            let mut rng = rand::rng();

            // Fake some work by sleeping for a random duration.
            let sleep_ms = rng.random_range(1000..2000);

            thread::sleep(Duration::from_millis(sleep_ms));

            // Create the message that will be sent to the logger.
            let msg = format!(
                "worker {}: task done after {}ms",
                i, sleep_ms
            );

            // Send the message through this worker's Sender.
            //
            // `tx1` belongs to this worker because of `move`.
            tx1.send(msg).unwrap();

            // When this thread finishes, tx1 is automatically dropped.
        });

        // Keep the JoinHandle so main can wait for this worker.
        handles.push(handle);
    }

    // ============================================================
    // 3. SPAWN THE LOGGER
    // ============================================================
    //
    // We start the logger BEFORE joining the workers.
    //
    // This makes sense because the logger can process messages
    // while workers are still doing their work.
    //

    let logger = thread::spawn(move || {
        // `rx` has been moved into the logger thread.
        //
        // The loop keeps receiving messages until the channel closes.
        //
        // IMPORTANT:
        // The channel closes only when ALL Sender handles are dropped.
        //

        for msg in rx {
            println!("[LOGGER] {msg}");
        }

        // We reach here only after every Sender is gone.
        println!("[LOGGER] Channel closed. Exiting.");
    });

    // ============================================================
    // 4. WAIT FOR ALL WORKERS
    // ============================================================
    //
    // `join()` has NOTHING to do with receiving messages.
    //
    // It simply means:
    //
    //     "Main, wait until this worker finishes."
    //
    // Since we stored the handles in the same order we created
    // them, we join them in that order.
    //

    for handle in handles {
        handle.join().unwrap();
    }

    // At this point:
    //
    //     Worker 0 -> finished -> tx1 dropped
    //     Worker 1 -> finished -> tx1 dropped
    //     Worker 2 -> finished -> tx1 dropped
    //
    // BUT main still owns the ORIGINAL `tx`.
    //
    // Therefore the channel is NOT closed yet.

    // ============================================================
    // 5. DROP MAIN'S SENDER
    // ============================================================
    //
    // Now we remove the final Sender.
    //
    // All Sender handles are gone:
    //
    //     tx       -> dropped
    //     tx1 #0   -> dropped
    //     tx1 #1   -> dropped
    //     tx1 #2   -> dropped
    //
    // Therefore the channel becomes disconnected.
    //

    drop(tx);

    // The logger's:
    //
    //     for msg in rx
    //
    // will now finish once all already-sent messages have been
    // received.
    //

    // ============================================================
    // 6. WAIT FOR THE LOGGER
    // ============================================================
    //
    // The logger has to finish processing the messages and exit
    // before main exits.
    //

    logger.join().unwrap();

    println!("Main exiting.");
}
```

```
Workers finish
      ↓
Their tx clones drop
      ↓
main still has tx
      ↓
drop(tx)
      ↓
NO SENDERS LEFT
      ↓
channel closes
      ↓
logger's `for msg in rx` ends
      ↓
logger exits
      ↓
logger.join()
      ↓
main exits
```
