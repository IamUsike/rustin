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

- Write a `TokenBucket` (capacity, refill rate, tokens, last refill) with `fn try_acquire(&mut self, now: Instant) -> bool`. Refill lazily.
- Write a `SlidingWindowCounter` (window, limit, prev count, current count, current window start) with the same method. Use `count = cur + prev * (1 - elapsed / window)`.
- Both have the same method signature, so a trait `Limiter` with 2 implementations is justified here (TRPL ch 10/18).
- Pass `now` in as an argument instead of calling `Instant::now()` inside, so tests can fake time.

**Check:** unit tests for each.
- A bucket of capacity 5 allows 5 calls and rejects the 6th.
- After waiting refill time, it allows again.
- The sliding window counter rejects once `limit` is reached, and the weighted previous window counts.

### Stage 2: Concurrent, per key (tokio)

- Wrap the limiters in `HashMap<String, TokenBucket>` behind `Arc<Mutex<...>>`. The key is the IP or user ID.
- Spawn many tasks that hit the same key concurrently.
- Question to answer in a comment: `std::sync::Mutex` or `tokio::sync::Mutex`? Why? (See your mini-redis `shared_state` notes.)

**Check:** 100 tasks hit one key with limit 10. Exactly 10 succeed.

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
