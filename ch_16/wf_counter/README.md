## Channels vs shared state — implement the same thing twice

Build a concurrent word frequency counter two ways. Input: 4 strings (hardcoded). Version A: use channels — each thread sends a HashMap<String,u32> back to main, main merges them. Version B: use Arc<Mutex<HashMap>> — threads directly update the shared map. Make both produce identical output. Then write 3 bullet points on when you'd choose each approach and why.

> Cements: channels = message passing (no shared state), Mutex = shared state (careful locking). Both valid, different tradeoffs.
