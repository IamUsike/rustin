# Rate Limiter

**Question:** Design and build a rate limiter for an HTTP API.

The full design came out of a mock system design interview. See `INTERVIEW_LOG.md` and `design/sys_design.v2.png`.

## Final design (summary)

```
client -> L4 LB -> IP limiter (token bucket, Redis A) -> L7 LB / gateway
       -> user limiter (verify JWT / API key, sliding window counter, Redis B)
       -> internal LB -> app servers
```

- **IP tier:** token bucket per client IP, lenient, absorbs bursts. Atomic via a Lua script in Redis.
- **User tier:** verify the JWT (cookie or `Authorization: Bearer`) or API key, then a sliding window counter per user per rule. `POST /login` is limited per IP (from `X-Forwarded-For`).
- **Rules:** `rules.toml`, reloaded at runtime without a restart.
- **Responses:** `429` + `Retry-After` + `X-RateLimit-Limit/Remaining/Reset`. `401` for a bad token. `503` when overloaded.
- **Redis down:** fall back to an in-memory limiter at `limit / N` per instance, fail open, log loudly.

## What you build locally

A scaled-down version that has the same ideas but runs on a laptop:

- **One** axum binary with **both** tiers stacked as two middlewares (IP first, then user).
- Redis in Docker.
- Later: `docker compose` with nginx in front of 2 copies of the binary, so you can prove the limit holds across instances.

## Stages

Do them in order. Each stage ends with a check you can run. Don't move on until it passes.

### Stage 1: Algorithms in memory (std only)

Build these to exactly this API so the tests below compile:

```rust
pub trait Limiter {
    fn try_acquire(&mut self, now: Instant) -> bool;
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_per_sec: f64, now: Instant) -> Self;
}

impl SlidingWindowCounter {
    pub fn new(limit: u32, window: Duration, now: Instant) -> Self;
}
```

Rules the tests assume:

- **TokenBucket** starts full. It refills lazily, at `refill_per_sec` tokens per second, inside `try_acquire`. Tokens are fractional, so half a token is kept and not lost. Tokens never go above `capacity`.
- **SlidingWindowCounter**: windows are aligned to the `now` passed to `new`, so they are `[t0, t0+w)`, `[t0+w, t0+2w)` and so on. A request is allowed if `cur + prev * (1 - elapsed_in_cur / w) < limit`, and then `cur += 1`. Rejected requests are not counted. Traffic older than the previous window is forgotten.
- `now` is passed in instead of calling `Instant::now()` inside, so tests can fake time.

**Check:** paste this at the bottom of the file that defines the limiters, then run `cargo test`. All 14 must pass.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn sec(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    // Calls try_acquire `tries` times at the same instant, returns how many were allowed.
    fn count_allowed(l: &mut impl Limiter, now: Instant, tries: u32) -> u32 {
        (0..tries).filter(|_| l.try_acquire(now)).count() as u32
    }

    // ---------- TokenBucket ----------

    #[test]
    fn bucket_starts_full_and_rejects_when_empty() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);

        assert_eq!(count_allowed(&mut b, t1, 10), 5);
    }

    #[test]
    fn bucket_refills_over_time() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);
        count_allowed(&mut b, t0, 5);
        assert_eq!(count_allowed(&mut b, t0 + sec(1), 10), 1);
        assert_eq!(count_allowed(&mut b, t0 + sec(3), 10), 2);
    }

    #[test]
    fn bucket_keeps_partial_tokens() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);
        count_allowed(&mut b, t0, 5);
        assert!(!b.try_acquire(t0 + ms(500))); // half a token: not enough
        assert!(b.try_acquire(t0 + ms(1000))); // that half must not be thrown away
    }

    #[test]
    fn bucket_never_exceeds_capacity() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);
        assert_eq!(count_allowed(&mut b, t0 + sec(3600), 100), 5);
    }

    #[test]
    fn bucket_fractional_refill_rate() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(2, 0.5, t0); // 1 token every 2 seconds
        count_allowed(&mut b, t0, 2);
        assert_eq!(count_allowed(&mut b, t0 + sec(1), 5), 0);
        assert_eq!(count_allowed(&mut b, t0 + sec(2), 5), 1);
    }

    // ---------- SlidingWindowCounter ----------

    #[test]
    fn window_allows_limit_then_rejects() {
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        assert_eq!(count_allowed(&mut w, t0, 20), 10);
    }

    #[test]
    fn window_start_of_next_window_still_full() {
        // at exactly t=60s the previous window has weight 1.0
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0, 10);
        assert_eq!(count_allowed(&mut w, t0 + sec(60), 10), 0);
    }

    #[test]
    fn window_weights_previous_window() {
        // halfway into window 1: estimate = cur + 10 * 0.5 = cur + 5
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0, 10);
        assert_eq!(count_allowed(&mut w, t0 + sec(90), 20), 5);
    }

    #[test]
    fn window_weight_uses_position_in_current_window() {
        // 15s into window 1: weight 0.75, estimate = cur + 7.5, so cur 0, 1, 2 pass
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0 + sec(10), 5);
        count_allowed(&mut w, t0 + sec(50), 5);
        assert_eq!(count_allowed(&mut w, t0 + sec(75), 20), 3);
    }

    #[test]
    fn window_rejected_requests_do_not_count() {
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        assert_eq!(count_allowed(&mut w, t0, 15), 10); // 5 rejected
        assert_eq!(count_allowed(&mut w, t0 + sec(90), 20), 5); // prev is 10, not 15
    }

    #[test]
    fn window_forgets_traffic_two_windows_old() {
        // at t=125s, current window is [120, 180), previous [60, 120) was empty
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0, 10);
        assert_eq!(count_allowed(&mut w, t0 + sec(125), 20), 10);
    }

    #[test]
    fn window_skips_idle_windows_without_drift() {
        // idle for many windows, then traffic: windows must stay aligned to t0
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0 + sec(600), 10); // window [600, 660)
        assert_eq!(count_allowed(&mut w, t0 + sec(690), 20), 5); // halfway into [660, 720)
    }

    #[test]
    fn window_late_first_request_does_not_shift_windows() {
        // windows stay [0,60) [60,120) [120,180) no matter when requests arrive
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        assert_eq!(count_allowed(&mut w, t0 + sec(59), 10), 10); // [0,60) = 10
        // t=100: 40s into [60,120), prev weight 1/3, estimate = cur + 3.33
        assert_eq!(count_allowed(&mut w, t0 + sec(100), 20), 7);
        // t=150: 30s into [120,180), prev = 7 (from [60,120)), weight 0.5, estimate = cur + 3.5
        assert_eq!(count_allowed(&mut w, t0 + sec(150), 20), 7);
    }

    #[test]
    fn bucket_out_of_order_timestamps_do_not_mint_tokens() {
        // Stage 2 preview: with many tasks, an older `now` can arrive after a newer one
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);
        count_allowed(&mut b, t0 + sec(10), 5); // drained at t=10
        b.try_acquire(t0 + sec(9)); // late, older timestamp
        assert_eq!(count_allowed(&mut b, t0 + sec(10), 5), 0); // no time passed since t=10
    }
}
```

### Stage 2: Concurrent, per key (tokio)

One limiter per key (IP or user ID), shared by many tokio tasks.

Setup: in `Cargo.toml`, set `tokio = { version = "1", features = ["full"] }`. The tests need the `macros` and `rt-multi-thread` features.

Build this in `src/keyed.rs`, then add `mod keyed; pub use keyed::RateLimiter;` to `lib.rs`:

```rust
pub struct RateLimiter<L> {
    // a Mutex around a HashMap<String, L>
    // a boxed closure that builds a new L for a key seen for the first time:
    //     Box<dyn Fn(Instant) -> L + Send + Sync>
}

impl<L: Limiter> RateLimiter<L> {
    pub fn new(make: impl Fn(Instant) -> L + Send + Sync + 'static) -> Self;
    pub fn check(&self, key: &str) -> bool;   // a normal fn, not async
}
```

Rules the tests assume:

- `check` creates the key's limiter on first use with `make(now)`, then calls `try_acquire(now)`.
- Read `Instant::now()` **while holding the lock**. Stage 1's last test explains why.
- `check` takes `&self`, not `&mut self`. That's the point of the Mutex: an `Arc<RateLimiter<_>>` gets shared across tasks, and they can't each get `&mut`.

Questions to answer in comments:

1. `std::sync::Mutex` or `tokio::sync::Mutex`? Why? (See your mini-redis `shared_state` notes. Hint: is there an `.await` while the lock is held?)
2. Why must the closure be `Send + Sync + 'static`? What breaks if you remove each one?
3. Why `Box<dyn Fn>` instead of a second generic parameter `F`? (TRPL ch 18. Think about how long the type name gets when Stage 3 stores this in axum `State`.)
4. The map never removes keys. What happens with 10M users? How would you fix it? (Don't fix it yet. In Stage 4, Redis TTLs do it for you.)

**Check:** paste this at the bottom of `src/keyed.rs` and run `cargo test`. All 5 must pass, plus the 14 from Stage 1.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SlidingWindowCounter, TokenBucket};
    use std::sync::Arc;
    use std::time::Duration;

    // Spawns `tasks` tokio tasks that all call `check(key)` once. Returns how many were allowed.
    async fn hammer<L: Limiter + Send + 'static>(rl: &Arc<RateLimiter<L>>, key: &str, tasks: u32) -> u32 {
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
```

### Stage 3: HTTP middleware (axum)

Read first: axum `Router`, handlers, `State`, and `axum::middleware::from_fn_with_state`. Skip raw tower `Service` for now.

- Use one dummy route, `GET /hello`, plus `POST /login`.
- IP middleware: get the client IP (`ConnectInfo<SocketAddr>`, or `X-Forwarded-For` if present) and run the token bucket.
- User middleware: read the JWT from the cookie or the `Authorization: Bearer` header, and verify it (`jsonwebtoken` crate, HS256 is fine locally). A bad token returns `401`. A good token goes through the sliding window counter keyed by the `sub` claim.
- When limited, return `429` with `Retry-After` and the `X-RateLimit-*` headers.

**Check:** `for i in $(seq 1 120); do curl -s -o /dev/null -w "%{http_code}\n" ...; done | sort | uniq -c` shows the expected 200/429 split. A forged token returns 401.

### Stage 4: Redis

Read first: Redis `INCR`, `EXPIRE`, `GET`, `EVAL`/`EVALSHA`, and why Lua scripts are atomic. Run `redis-cli` and try the commands by hand before writing Rust.

- Use the `redis` crate (tokio feature) with a connection manager in `State`.
- Token bucket goes in a Lua script: read the fields, refill, decide, write, set the TTL, all in one `EVALSHA`.
- Sliding window counter: `INCR` the current-window key plus `EXPIRE`, then `GET` the previous-window key.
- Keys look like `rl:ip:<ip>` and `rl:u:<user>:<rule>`. Every key gets a TTL.

**Check:** run 2 copies of the binary on different ports against the same Redis. Hit both alternately. The combined total stays at the limit.

### Stage 5: Dynamic rules

- Write `rules.toml` with per-route limits (route, key type `ip` or `user`, algorithm, limit, window). Parse it with `serde` + `toml`.
- Reload it at runtime: poll the file's modified time every few seconds in a spawned task, or use the `notify` crate. Swap the rules in with an `Arc<RwLock<Rules>>` (or look up `arc-swap`).

**Check:** change a limit in the file while the server runs. The new limit applies without a restart.

### Stage 6: Redis down fallback

- If a Redis call errors or exceeds a timeout (about 5 ms, `tokio::time::timeout`), use the Stage 2 in-memory limiter with `limit / N`. N is an env var.
- Log a warning (`tracing`).

**Check:** run `docker stop redis` mid-test. Requests are still limited, just locally. Bring Redis back and they go back to the shared limit.

### Stage 7: Multi-instance and load test

- Write a `docker compose` file with `redis`, 2 copies of the app, and nginx load-balancing them. nginx sets `X-Forwarded-For`.
- Load test with `oha` (e.g. `oha -z 30s -c 50 http://localhost:8080/hello`).

**Check:** the 429 rate matches the configured limit across both instances. p99 latency added by the limiter stays in single-digit ms.

## Tools you'll pick up

axum, tower (lightly), tokio, Redis + Lua, `redis` crate, `jsonwebtoken`, `serde`/`toml`, `tracing`, Docker Compose, nginx, oha.

## Out of scope (on purpose)

Multi-region, separate binaries per tier, a real Kubernetes setup, CAPTCHA, and raw tower `Service`/`Layer` (an optional rewrite of the Stage 3 middleware later).
