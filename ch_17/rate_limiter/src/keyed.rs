use std::{collections::HashMap, sync::Mutex, time::Instant};

use crate::Limiter;

pub struct RateLimiter<L> {
    limiters: Mutex<HashMap<String, L>>, //stores the type of limiter
    make: Box<dyn Fn(Instant) -> L + Sync + Send>,
}

impl<L: Limiter> RateLimiter<L> {
    pub fn new(make: impl Fn(Instant) -> L + Send + Sync + 'static) -> Self {
        Self {
            limiters: Mutex::new(HashMap::new()),
            make: Box::new(make),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut map = self.limiters.lock().unwrap();
        let now = Instant::now();

        //if the particular key doesnt exist create a new entry
        //in the hashmap
        if !map.contains_key(key) {
            let limiter = (self.make)(now);
            map.insert(key.to_string(), limiter);
        }

        map.get_mut(key).unwrap().try_acquire(now)
    }
}

/* ANNSWERS
* 1. std::sync::Mutex or tokio::sync::Mutex? Why?
* -> Tokio mutex holds locks across awaits. (ie doesnt block the thread waiting for the lock).
* There's no await while the lock is held so we can use the std mutex. And it also mentions
* in the doc to use the tokio mutex only when theres a lot of contention, else it provides no
* real benefits (it uses the sync mutex internally).

* 2. Why must the closure be Send + Sync + 'static? What breaks if you remove each one?
* -> closures must be Send and Sync cos we need the rl to be accessed my multiple threads
* and also to be shared among threads. The 'static keyword means that all the data should
* be either owned by the thread(fn) or it should have 'static lifetime. this is true cos
* tasks can outlive the caller funtion. (but we arten't using tasks here, so i'm assuming
* that'll happen later).

* 3. Why Box<dyn Fn> instead of a second generic parameter F?
* -> because the types that are implementing the `limiter` trait are 2 and if we use a generic
* param only one type can be stored.

* 4. The map never removes keys. What happens with 10M users? How would you fix it?
* -> I would fix it by adding TTLs. Not entirely sure how to do it. will look it up at
* stage 4
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SlidingWindowCounter, TokenBucket};
    use std::sync::Arc;
    use std::time::Duration;

    // Spawns `tasks` tokio tasks that all call `check(key)` once. Returns how many were allowed.
    async fn hammer<L: Limiter + Send + 'static>(
        rl: &Arc<RateLimiter<L>>,
        key: &str,
        tasks: u32,
    ) -> u32 {
        let handles: Vec<_> = (0..tasks)
            .map(|_| {
                let rl = Arc::clone(rl);
                let key = key.to_string();
                tokio::spawn(async move { rl.check(&key) })
            })
            .collect();

        let mut allowed = 0;
        for h in handles {
            if h.await.unwrap() {
                allowed += 1;
            }
        }
        allowed
    }

    // Refill so slow it never adds a token during a test.
    fn bucket_of_10() -> RateLimiter<TokenBucket> {
        RateLimiter::new(|now| TokenBucket::new(10, 0.000_001, now))
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn one_key_many_tasks_exactly_limit_pass() {
        let rl = Arc::new(bucket_of_10());
        assert_eq!(hammer(&rl, "1.2.3.4", 100).await, 10);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn keys_are_independent() {
        let rl = Arc::new(bucket_of_10());
        assert_eq!(hammer(&rl, "alice", 50).await, 10);
        assert_eq!(hammer(&rl, "bob", 50).await, 10); // alice's traffic must not touch bob
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn many_keys_at_once() {
        // 50 keys hammered concurrently, 20 tasks each: every key gets exactly 10
        let rl = Arc::new(bucket_of_10());
        let handles: Vec<_> = (0..50)
            .map(|i| {
                let rl = Arc::clone(&rl);
                tokio::spawn(async move { hammer(&rl, &format!("user-{i}"), 20).await })
            })
            .collect();

        for h in handles {
            assert_eq!(h.await.unwrap(), 10);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn works_with_sliding_window_too() {
        let rl = Arc::new(RateLimiter::new(|now| {
            SlidingWindowCounter::new(10, Duration::from_secs(60), now)
        }));
        assert_eq!(hammer(&rl, "1.2.3.4", 100).await, 10);
    }

    #[test]
    fn plain_sync_call_works() {
        // check() is a normal fn: no runtime needed to use it
        let rl = bucket_of_10();
        let allowed = (0..15).filter(|_| rl.check("k")).count();
        assert_eq!(allowed, 10);
    }
}
