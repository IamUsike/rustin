# minichat: build guide

An axum WebSocket relay and a CLI client exchanging end-to-end encrypted 1:1 messages.
The owner is learning axum by building this. Tower is out of scope; boilerplate is fine there.

## How Claude helps in this repo

- Do not write implementation code, even small snippets, unless explicitly asked.
- Answer with direction: which file, which type or doc page, what to think about.
- Let the owner attempt first. Correct after they commit to an answer.
- On a review, point at the flaw and ask the question that exposes it. Do not hand over the patch.
- Boilerplate is the exception: Cargo manifests and tower/tracing setup can be given directly.

## Scope

In: register, 1:1 messages, E2EE, offline delivery, key fingerprint check.
Out: groups, media, persistence, Tor, forward secrecy.

## Layout

A workspace of three crates. Create each file at the milestone that needs it, not before.

```
message_app/
├── Cargo.toml             # [workspace] only, shared versions in [workspace.dependencies]
└── crates/
    ├── proto/src/lib.rs   # ClientMsg, ServerMsg. Serde only, no logic.
    ├── server/
    │   ├── src/main.rs    # bind and serve, nothing else
    │   ├── src/lib.rs     # pub fn app(state) -> Router, so tests can start the server
    │   ├── src/state.rs   # AppState, UserEntry, and every rule that touches the map
    │   ├── src/ws.rs      # upgrade handler, per-connection tasks, ClientMsg dispatch
    │   └── tests/relay.rs
    └── client/
        └── src/
            ├── main.rs    # clap, connect, the select! loop
            ├── crypto.rs  # pure functions on bytes: no I/O, no async
            └── store.rs   # identity key file, known contacts file (TOFU)
```

The root `Cargo.toml` is currently a single package. Convert it, and delete the root `src/`.

### Why these boundaries

- **`proto` is a separate crate** because both binaries must agree on one type, not on two copies.
- **`crypto` lives only in the client.** The server cannot depend on it, so "the server never sees
  plaintext" holds by construction. Keep it that way.
- **`state.rs` methods are synchronous.** A plain `fn` cannot hold a guard across an `.await`, so
  the classic bug cannot be written. `ws.rs` never locks anything directly.
- **`ws.rs` knows sockets, `state.rs` knows rules.** If `state.rs` imports a WebSocket type, the
  split has gone wrong.
- **`lib.rs` plus `main.rs` in the server** exists so integration tests can start a real server.
- **`crypto.rs` is pure** so tamper tests need no network or files.

Do not add traits, a `handlers/` tree, or an error-type hierarchy. `anyhow` covers this project.

## Crates

Add with `cargo add` to get current versions. Checked 2026-11-02: axum 0.8, tokio-tungstenite 0.30,
x25519-dalek 3.0, chacha20poly1305 0.11, hkdf 0.13, sha2 0.11, rand 0.10.

The original list misses or understates these:

- `futures-util` is needed for `.split()` and for `StreamExt`/`SinkExt` on both sockets.
- x25519-dalek needs feature `static_secrets`. `EphemeralSecret` cannot be saved to a file.
- x25519-dalek feature `getrandom` provides key generation without passing an RNG.
- These crate majors are recent. Blog posts will be wrong, so read docs.rs for the exact version.
  If an RNG trait bound fails with a confusing error, suspect a `rand_core` version mismatch.
  Run `cargo tree -d`. Tungstenite already brings in an older copy, so seeing two is normal.

## Protocol holes to fix

The original spec has these problems. Each is yours to solve, at the milestone shown.

1. **Reconnecting is impossible as specified (milestone 5).** Bob goes offline, comes back, sends
   `Register`, and gets `Taken`. Offline delivery depends on solving this. The obvious fix is "same
   pubkey means same user", but pubkeys are public: what can an attacker then do? X25519 cannot
   sign. Decide what you accept for v1, and record the ceiling in a comment.
2. **`NotFound` has no username (milestone 2).** With two lookups in flight, which one failed?
3. **`Send` to an unknown user, or before registering (milestone 3).** Neither case has a reply.
   Decide what happens, and what malformed JSON does to the connection.
4. **The sender can be spoofed unless you are careful (milestone 3).** `Send` correctly has no
   `from` field. Fill `Incoming.from` from per-connection state. Never trust the client for it.
5. **`oneshot` cannot test WebSockets (milestone 7).** It never completes an HTTP upgrade. Bind
   `127.0.0.1:0`, spawn `axum::serve`, and connect with tokio-tungstenite.

## Milestones

### 1. Echo

Done when a client connects over WS and the server echoes text back.
Files: `server/main.rs`, `client/main.rs`. Read: `axum::extract::ws`, and the axum `websockets` example.
Test it with `websocat` first, so you debug one side at a time. Axum and tungstenite have separate
`Message` types. Handle `Close`, and notice who answers `Ping`.

### 2. Register and lookup

Done when two clients register, `lookup bob` returns his key, and duplicates are rejected.
Files: `proto`, `state.rs`, `ws.rs`.
Think about: `serde(tag = ...)` on the enums; which of serde's enum representations you want to read
in logs; how `Vec<u8>` looks in JSON and whether you care yet. Check key length at the server boundary.

Use `std::sync::Mutex`, not tokio's: critical sections are short and contain no `.await`. Read the
tokio docs section on which mutex to use. Check-then-insert must happen under one lock acquisition.

### 3. Plaintext relay

Done when `send bob hi` appears on Bob's terminal.
Server: split the socket into a reader task and a writer task. The writer drains an `mpsc` of
`ServerMsg`, not of raw frames. Work out what stops the other task when one ends, and put map cleanup
in exactly one place.

Client: `select!` on stdin and socket.
- Read the cancel-safety notes. Know why `lines().next_line()` is safe here and `read_line` is not.
- The tokio stdin docs recommend a dedicated thread for interactive input. Know why before you skip it.
- Never wait for a reply inline. `send bob hi` needs Bob's key, but the answer arrives on the same
  loop that must also keep receiving. Which state holds a message waiting for a key?

### 4. Encryption

Done when server logs show only ciphertext and Bob still reads the message.
Files: `crypto.rs`, `store.rs`, plus changes in `client/main.rs`. Server code should not change at all.
If it does, ask why.

Write `crypto.rs` with its tests before wiring it in. Questions to answer in that file:
- Alice and Bob must derive the same key. What goes in HKDF's `info`, and in what order?
- Both directions share one key. The server takes Alice's ciphertext to Bob and returns it to Alice
  as "from Bob". It decrypts. Fix this using AEAD associated data.
- What does `was_contributory()` protect against? Should you check it?
- Random nonces are fine here. What is the limit, and why is reuse catastrophic for this cipher?
- Bob receives a message from Alice but has no Alice key yet. This is the waiting problem from
  milestone 3, in the receiving direction.

Add a `--data-dir` flag early: two clients on one machine must not share an identity. Create the key
file with mode `0600`, and keep it out of git.

### 5. Offline mailbox

Done when a message sent while Bob is offline arrives on his next connect.
Store whole `ServerMsg::Incoming` values, so replay is the same code path as live delivery.

Races to reason through:
- Bob reconnects before the old connection's cleanup runs. The old cleanup must not erase the new
  sender. Look at `Sender::same_channel`, or use a connection id.
- The send to the live channel fails because Bob just dropped. The message must go to the mailbox.
- A message arrives during replay. Can it overtake older ones?
- Bob's writer dies with messages still in the channel. They are lost without acks. Accept this for
  v1, and mark it with a comment.
- Anyone can fill Bob's mailbox. Cap it, and decide what overflow does. Check the WebSocket max
  message size too.

### 6. Fingerprints and TOFU

Done when a changed key triggers a loud warning.
Files: `store.rs`, `client/main.rs`. The fingerprint must be identical on both sides, whichever way you
pass the keys. Check on every `PubKey` reply, before encrypting. Does a warning still send the message,
or does it hold it until the user acts? Test it by deleting a client's key file and re-registering,
which also tests your milestone 5 decision.

### 7. Tests

Done when integration tests drive the server over a real socket, and a crypto round-trip test covers
tamper detection (flip a bit, expect decryption failure).

Minimum set: duplicate register; lookup miss; relay shows the correct `from`; offline replay in order;
reconnect race; wrong key fails; flipped ciphertext bit fails; swapped associated data fails.

## Known limits (acceptable for v1)

- Static-static DH: one leaked key exposes all past messages. No forward secrecy.
- The server can replay stored ciphertext. Nothing in this protocol detects that.
- Hand-composed primitives are for learning. The real app should use X3DH + Double Ratchet, or Noise.

## Commands

```
cargo run -p server
cargo run -p client -- --data-dir /tmp/alice
cargo test --workspace
cargo clippy --workspace
```

The crate names and flags above assume the layout in this guide. Update them if yours differ.
