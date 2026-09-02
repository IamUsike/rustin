# async fn is just syntax sugar

Write the same function two ways: (A) as an async fn that sleeps 1 second and returns a String. (B) as a regular fn that returns impl Future<Output=String> using async move {} block. Verify both compile and behave identically. Then in comments, write out what the compiler approximately desugars (A) into — a state machine with states: Start, Sleeping, Done.

---

when exec reaches:

```rust
aa().await;
```

roughly this happens:

```
main task
   |
   v
poll aa()
   |
   v
aa reaches sleep(...).await
   |
   v
sleep isn't ready
   |
   v
aa returns Pending
   |
   v
main also returns Pending
   |
   v
control goes back to Tokio runtime
```

main does NOT continue to println!("main after a").

The runtime can now execute some other task that is ready. Later, when the timer expires, Tokio wakes the main task and polls it again:

```
aa().await;
println!("A");

bb().await;
println!("B");
```
