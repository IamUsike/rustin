```rust
fn main() {
    let (tx, rx) = mpsc::channel();

    for i in 0..5 {
        tx.send(i).unwrap();
    }

    let handle = thread::spawn(move || {
        for r in rx {
            println!("{r}");
        }
    });

    drop(tx); //else main owns tx and the sender keeps waiting
    handle.join().unwrap();
}
```

- tx remains in scope because main still owns it.
- `join()` blocks main, so main doesn't reach the end of its scope and automatically drop tx.
- Therefore the receiver waits forever.

If you removed `join()` entirely, main could exit and the whole process would terminate, potentially before the spawned thread finishes. So `join()` is doing the right thing; you just need to close the channel before waiting for the consumer.
