2. std::sync::Mutex gives you no fairness guarantee.

There's no queueing/FIFO promise about who gets a lock next when it's released. Combine that with point 1 (one thread holds the lock for a long critical section, then very quickly loops back around and tries to grab it again) — think about what's likely to happen to a thread that just released a lock and immediately asks for it again, versus threads that have been parked waiting for a while. This is a well-known phenomenon with a name — might be worth looking up "lock convoy" or "thread starvation."

go through the below info about atomics and data access
https://doc.rust-lang.org/beta/nomicon/atomics.html
