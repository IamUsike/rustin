- questions and answers

1. Do you want the rl to be client side or server side ?

> Server side. We own the API, and clients can't be trusted to throttle themselves

2. how much traffic are we handling? Who is being rate limited? what params are we using mto rate limit? What happens when we rl someone? Also where does this run? like in an api gateway/middleware or in the server code?

> Interviewer answers:

1. Traffic: 10M daily active users. Peak is about 50k requests/sec across the whole API. We run 20 API server instances behind a load balancer.
2. Who: logged-in users are limited by user ID. Anonymous traffic is limited by IP.
3. Limits: they differ per endpoint. For example:
   - General API: 100 requests/min per user.
   - POST /login: 5 requests/min per IP.
4. When someone is limited: reject the request immediately, with no queueing. The client must be told it was limited and when it can retry.
5. Where it runs: your call. Pick one and justify it.

6. - What happens if the rate limiter itself goes down? Do we let traffic through (fail open) or block it (fail closed)?

- How much latency can the limiter add per request?
- Must the limit be exact, or is it OK if a user sometimes gets 105 instead of 100?
- Are bursts allowed? Can a user send all 100 requests in the first second?
- Can limits change without a redeploy?
- One region or several?

> Interviewer answers:

- Limiter down: fail open and let traffic through, but fire an alert. Availability matters more than strictness.
- Latency: at most 5ms added at p99.
- Regions: single region for now.
- Bursts: short bursts are OK as long as the per-minute limit holds.
- No redeploy: yes, limits must change at runtime. "Hot reloading" usually means reloading code. For config, say "dynamic config" or "runtime config reload".
- Exactness: approximate is fine. A few percent over is acceptable.
