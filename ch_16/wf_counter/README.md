## Channels vs shared state — implement the same thing twice

Build a concurrent word frequency counter two ways. Input: 4 strings (hardcoded). Version A: use channels — each thread sends a HashMap<String,u32> back to main, main merges them. Version B: use Arc<Mutex<HashMap>> — threads directly update the shared map. Make both produce identical output. Then write 3 bullet points on when you'd choose each approach and why.

> Cements: channels = message passing (no shared state), Mutex = shared state (careful locking). Both valid, different tradeoffs.

---

Your first point

> "when we want everything each counter to occur without any block, we can choose mpsc"

Not quite. Channels can still block — send() can block depending on the channel type, and recv() obviously waits for data.

A better reason:

Channels: Use when you want threads to produce independent results and send ownership of those results to another thread. It reduces direct shared-state synchronization.

Your second point

> "Arc has less code"

That's not really a good reason for choosing Arc<Mutex<_>>. In your particular implementation it may have less code, but that's not the fundamental reason.

Better:

- Arc<Mutex<_>>: Use when threads need to directly access and modify the same shared state, with the mutex ensuring only one thread modifies it at a time.

- You need 3 points, so you could write:

-> Channels: Good when threads produce independent results and you want to pass those results between threads without sharing mutable state.
-> Arc<Mutex<_>>: Good when multiple threads need to directly access and modify the same shared data.
-> Channels vs Arc<Mutex<_>>: Channels encourage message passing/ownership transfer, while Arc<Mutex<_>> is better suited when the shared state itself is the thing being collaboratively modified.
