# Rate Limiter: System Design Interview Log

Mock interview, 2026-10-18. Prompt given: **"Design a rate limiter for our API."**

Diagrams: `design/sys_design.v1.png` (my first attempt), `design/sys_design.v2.excalidraw` / `.png` (final).

---

## Phase 1: Clarifying questions

### What I asked and what I got back

| # | My question | Interviewer answer |
|---|---|---|
| 1 | Client side or server side? | Server side. We own the API; clients can't be trusted. |
| 2 | How much traffic? | 10M DAU, ~50k req/s peak, 20 API instances behind an LB. |
| 3 | Who is limited? | Logged-in users by user ID, anonymous traffic by IP. |
| 4 | What are the limits? | Per endpoint. General: 100 req/min per user. `POST /login`: 5 req/min per IP. |
| 5 | What happens when limited? | Reject immediately, no queueing. Tell the client when it can retry. |
| 6 | Where does it run? | My call. Justify it. |
| 7 | Limiter down: fail open or closed? | Fail open, but alert. Availability over strictness. |
| 8 | Latency budget? | ≤ 5 ms added at p99. |
| 9 | Single or multi region? | Single region for now. |
| 10 | Bursts allowed? | Yes, short bursts, as long as the per-minute limit holds. |
| 11 | Change limits without redeploy? | Yes. (Term: *dynamic / runtime config reload*. "Hot reload" usually means reloading code.) |
| 12 | Exact or approximate? | Approximate. A few % overshoot is fine. |

Questions 7–12 were missed in my first batch and only asked after the interviewer pointed them out.

### Why each question matters (how a different answer changes the design)

- **Client vs server side.** Client-side limiting is only a courtesy (SDK backoff). Server side is the only real enforcement. If the answer were "client side", the design would be an SDK with retry/backoff, with no infrastructure.
- **Traffic.** Decides whether one process with an in-memory `HashMap` is enough, or whether you need shared state (Redis) and sharding. At 500 req/s on one server, a `Mutex<HashMap>` is the whole design. At 50k req/s across 20 servers, it is not.
- **Who is limited (key).** Decides the key and where the limiter can sit. Limiting by IP can happen at the edge before auth. Limiting by user ID needs auth done first, so it must sit after (or do) token verification.
- **Per-endpoint limits.** Rules need the HTTP path, so the limiter must be L7 (parse HTTP). One global limit could live in an L4 device.
- **What happens when limited.** Reject vs queue are completely different systems. Queueing means a message queue and workers (a throttler, not a limiter). Rejecting means `429` plus retry hints.
- **Where it runs.** In-process middleware has no extra hop, but it is coupled to app deploys and language. A gateway tier is one place to own it and is language-agnostic, but it adds a hop and its own scaling and failure story.
- **Fail open vs closed.** Fail open means a Redis outage lets traffic through, so you need a degraded local fallback and alerts. Fail closed (e.g. a payments API) means a Redis outage is an API outage, so Redis HA becomes critical-path.
- **Latency budget.** 5 ms rules out anything that calls a remote database per request, and allows one Redis round trip (~sub-ms in-region). A 50 ms budget would allow a lot more.
- **Regions.** Multi-region means either per-region limits (simple, but a user can get N× the limit) or cross-region replication of counters (slow, complex). Single region keeps one Redis per tier.
- **Bursts.** If bursts are allowed, use token bucket. If traffic must be smooth, use leaky bucket or sliding window.
- **Dynamic rules.** Rules must live outside the binary (a config file or store), with a reload mechanism.
- **Exactness.** "Approximate OK" allows the sliding window counter (cheap, O(1)), async Redis replication (a few counts lost on failover), and local fallback. "Exact" pushes you to the sliding window log (memory heavy) and strong consistency.

---

## Phase 2: Placement and architecture

| Round | My answer | What was wrong | Correct / final answer |
|---|---|---|---|
| Placement | Put the limiter "as middleware, before the LB", because many servers would need a shared counter. | A limiter tier with redundancy *also* needs a shared counter, so shared state doesn't decide placement. "Middleware" (in-process) and "before the LB" (separate tier) are different things. | Either placement works with central Redis. Choose on tradeoffs: an extra hop vs in-process; who owns and deploys it; language. Alex Xu's "rate limiter middleware" names a *role* (between client and servers), usually implemented in an API gateway. |
| Scaling | "Autoscale the load balancers using HPA." | HPA scales pods (a Deployment), not LBs. CPU-based HPA takes 30 s to minutes, but spikes take under 1 s. | Limiter pods are scaled by HPA, **plus headroom**: run at ~50–60% at peak so spikes fit while HPA catches up. |
| Pod count | 3 pods × 25k = 75k, plus 1 idle standby. Excess requests get queued. | With 1 pod dead, 2 × 25k = 50k means 100% load and no headroom. A standby is paid for and idle, and needs failover-detection time. Queuing breaks the 5 ms budget. | **4 active pods** (100k capacity, 50% at peak). Overload means **load shedding**: fast `503`, never a queue. |
| Placement v2 | Two tiers: an IP limiter at the edge, a user limiter after auth. | (Good. This is a real pattern.) | Kept. IP tier at the edge, user/API-key tier at the gateway. |
| Redis on limiter box | Run Redis on the same machine as the IP limiter. | With several limiter pods, each local Redis sees only part of the traffic (counts not shared). One machine is a SPOF. | One shared **Redis Cluster per tier**, separate from the limiter pods. |
| Ops vs requests | "Redis does 100k ops/s, so 50k req/s is fine." | One check is several Redis commands (sliding log: ZREMRANGEBYSCORE, ZADD, ZCARD, EXPIRE = 4). 4 × 50k = 200k ops/s. | Count **commands per check**, not requests. Use a Lua script (1 round trip). Shard so each node stays under ~100k ops/s. |
| Sharding | "hash(key) mod number_of_shards." | `mod N` remaps almost every key when N changes. | Redis Cluster: **16384 hash slots**, `CRC16(key) mod 16384`. Each node owns slot ranges, and resharding moves only some slots. |
| Shard dies | "Users on it reset; reshard to other shards." | Manual resharding isn't needed. | Each primary has a **replica**. Redis Cluster auto-promotes it in seconds. Replication is async, so a few counts are lost, which is fine because approximate is OK. |
| Atomicity | Lua script, or INCR-then-check. | (Both correct.) Nuance: INCR-then-check works only for counter algorithms. | Token bucket is read → compute → write, so it **needs Lua**. The sliding window counter can use `INCR` + `EXPIRE` (INCR is atomic). |
| Redis down | Serve from a local cache until Redis is back. (v1 drawing also had a message queue.) | With N pods, each local counter sees ~1/N of a user's traffic, so the effective limit becomes N×. The message queue is meaningless: decisions must be made *now*. | Local fallback with **`limit / N` per pod**, fail open, alert. Replica promotion means the fallback covers only seconds. Queue removed. |
| Request path | client → limiter → LB → gateway → limiter → app. | Nothing spreads traffic across the first limiter pods. The 20 app servers need balancing too. Behind an LB, the TCP source IP is the LB's. | client → **L4 LB (PROXY protocol)** → IP limiter (sets `X-Forwarded-For`) → **L7 LB / gateway** → user limiter → **internal LB** → 20 app servers. Trust `X-Forwarded-For` only when it is set by your own proxy. |
| L4 vs L7 | The "network" tier enforces `/login` limits. | An L4 device can't see HTTP paths. | The IP limiter pods are L7 (they parse HTTP). The `/login` per-IP rule lives in the user tier, using the IP from `X-Forwarded-For`. |

---

## Phase 3: Auth and identity

| Question | My answer | Correction |
|---|---|---|
| Where does the user ID come from? | From the cookie. | A cookie is client-controlled. A raw user ID in it can be forged. |
| What's in the cookie? (asked 3×) | (Deflected: "answers 1 and 2 cover this".) Then: verified after the gateway. Finally: a JWT, verified by the limiter. | **Deflecting is a red flag.** Final: it's a **JWT**. The limiter verifies the signature with a public key and checks `exp`. No DB lookup, so it fits 5 ms. The other common option is an opaque **session ID**, which needs a lookup in a session store. Verification must happen **before** limiting by user ID. |
| Mobile apps / scripts? | Scripts are limited by IP. Client fingerprinting is bypassable, so maybe CAPTCHA. | Mobile apps log in and send the JWT as `Authorization: Bearer <jwt>`. Same limiter, read the cookie *or* the header. Scripts/integrations get **API keys**, limited per key. Anonymous traffic gets the IP limit plus CAPTCHA/challenge (the CAPTCHA idea was right). The IP limit must stay lenient because of **carrier-grade NAT** (thousands of users on one IP). |

---

## Phase 4: Algorithms and memory

| Topic | My answer | Correction |
|---|---|---|
| IP tier | Token bucket: allows bursts, O(1) memory. Fields: tokens and last refill, refilled lazily. | Correct. Needs Lua for atomicity. |
| User tier | Sliding window counter: O(1) per user, some overshoot OK, "allows bursts". | Algorithm OK, reason wrong. Burst-friendliness is token bucket's selling point. The sliding window counter *smooths*. Better reason: the edge wants cheap burst absorption, the user tier wants fair per-minute quotas. |
| Sliding window counter error | Assumes the previous window was evenly distributed. Wrong when prev traffic is bunched at its end. Formula garbled. | Estimate: `count = cur + prev × (1 − elapsed_in_cur / w)`. Under-counts when previous-window traffic is bunched near its end. Cloudflare measured ~0.003% of requests wrongly allowed/limited on real traffic. |
| Memory | "5 KB avg request… 10M × 16 bytes." | Request size is irrelevant: you store counters, not requests. Raw ≈ 8–16 B/user, but Redis per-key overhead (key string + internals) is ~60–100 B. **10M × ~100 B ≈ 1 GB**, × number of rules. TTLs mean only users active in the current window are stored. **Memory fits one node: shard for throughput, not memory.** |

Other algorithms to know: **fixed window counter** (simplest, allows 2× bursts at window edges), **sliding window log** (exact, stores every timestamp, so memory is O(requests)), **leaky bucket** (smooth constant output rate, queue-based).

---

## Phase 5: Final two questions

| Question | My answer | Correction |
|---|---|---|
| Where do rules live (no redeploy)? | A config file on disk, reloaded dynamically. | Close. With 8 limiter pods, "on disk" means 8 disks to keep in sync. Use a **Kubernetes ConfigMap** mounted as `rules.toml` in every pod. The pod watches the file (or polls it) and atomically swaps the in-memory rules. The alternative is rules stored in Redis/a config service, polled every few seconds. |
| What does a limited client receive? | Header `x-retry-after`. | The standard header is **`Retry-After`** (seconds, RFC 9110), with no `x-` prefix. Also send `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` (or the newer IETF `RateLimit` / `RateLimit-Policy` headers). Status `429 Too Many Requests`. |

### Follow-up (2026-10-20): why the L4 LB, and where TLS ends

| Question | My answer | Correction |
|---|---|---|
| Why is there an L4 LB before the IP limiter? | (Asked by me.) | 4 limiter pods need one stable address, traffic spread across them, and health checks. DNS round-robin is cached and has no health checks. L4 only forwards TCP, so it is cheap and the least likely box to fall over. PROXY protocol passes the real client IP to the pods. |
| Where is TLS decrypted? | "At the L4 LB… HTTP headers aren't encrypted… so at the L7 LB?" | **HTTP headers *are* encrypted.** TLS encrypts the whole HTTP message: method, path, headers, body. Only the IP/TCP headers (IPs, ports) and the SNI hostname are visible. So TLS must end **before the first component that reads HTTP**, which is the IP limiter, not the later L7 gateway. Two valid options: the L4 LB terminates TLS (cloud-managed certs, pods get plain HTTP), or TLS passthrough with the IP limiter pods terminating it (certs in pods, TLS CPU cost in pods). |
| After decrypting at the LB, does the LB take the limiter's key and re-encrypt? Then the limiter decrypts anyway? | (My question.) | The LB never gets the limiter's key. Each hop is a **separate TLS session**: the LB opens a new TLS connection to the pod, the pod presents its own cert (from an internal CA), and the handshake makes fresh session keys. Yes, the limiter decrypts either way. So for a **pure L4 LB, re-encryption is pointless**: passthrough gives the same security with one fewer decryption. Re-encrypting is only worth it when the LB does something with the plaintext (L7 inspection, WAF) or you want public certs centralised at the LB. |
| Final TLS choice | | **Passthrough: the IP limiter terminates TLS.** Internal hops (limiter → gateway → user limiter → app) use **mTLS**, where both sides present certs, so each service knows who is calling it. In practice a **service mesh** (Istio, Linkerd) does mTLS in sidecar proxies, and app code stays plain HTTP. Cost is fine: the handshake is the expensive part (public-key crypto), bulk data uses AES (hardware accelerated), and keep-alive connections reuse one handshake for many requests. |

---

## How the diagram changed (v1 → v2)

| v1 | v2 | Why |
|---|---|---|
| Client connects straight to the IP limiter | Client → **L4 LB** → IP limiter | 4 pods need something to spread traffic across them. |
| "3 instances" | **4 active pods** per tier | Survive 1 pod loss with headroom. |
| Message queue "if redis is down" | Removed. Local `limit/N` fallback plus Redis replicas. | A limit decision can't wait in a queue. |
| JWT check shown only as a note, 429 only | Explicit **401** for a bad JWT, **503** for load shed | Different failures need different status codes. |
| One "Application server" box | **Internal LB → 20 app servers** | Matches the spec. |
| — | `X-Forwarded-For` set by the IP tier | The second tier needs the real client IP for `/login`. |
| — | **Rules ConfigMap** watched by both tiers | Dynamic rules requirement. |
| Redis clusters, unspecified | Cluster A: Lua token bucket. Cluster B: INCR/EXPIRE sliding window. Primaries + replicas. | Atomicity plus failover. |

---

## Interview technique notes (for next time)

- Ask clarifying questions **in one batch of 5–8**. Cover: who/what key, scale, behaviour on limit, failure mode (open/closed), latency, consistency/exactness, regions, config changes.
- **Never deflect** ("my earlier answers cover it"). If you don't know, say "I'm not sure. Let me reason it out" and think out loud.
- Count **operations**, not requests, when sizing a store.
- Pushback is the normal format, not a verdict. Interviewers grill good answers too. When grilled: pause, restate the question, and reason from first principles (what state exists, who reads/writes it, what happens when each box dies).
- Draw the full request path early, naming every hop. Most of the holes in this session were missing hops.
