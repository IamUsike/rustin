Scope: “minichat”

Two binaries (or one with subcommands) in a Cargo workspace:

server: an axum WebSocket relay. It never sees plaintext.
client: a CLI that registers a username, connects, and exchanges encrypted 1:1 messages.

In: register, 1:1 messages, E2EE, offline delivery, key fingerprint check.
Out (for now): groups, media, persistence, Tor, forward secrecy.

Architecture
client A ──ws──► server (routes by username, stores ciphertext) ──ws──► client B

The server holds Arc<Mutex<HashMap<String, UserEntry>>>, where each entry has a public key, an optional live mpsc::Sender, and a VecDeque mailbox of undelivered messages. Each WebSocket connection gets a task that reads frames and a task that drains its channel.

Wire protocol

Define one serde enum and send it as JSON over WebSocket:

ClientMsg: Register{username, pubkey} | Lookup{username} | Send{to, nonce, ciphertext}
ServerMsg: Registered | Taken | PubKey{username, pubkey} | NotFound | Incoming{from, nonce, ciphertext}
Crypto, minimal and sane
Each client generates an X25519 keypair on first run and stores it in a file.
To message Bob, fetch his public key from the server and compute the shared secret with X25519 DH.
Run it through HKDF-SHA256 to get a symmetric key, and encrypt with ChaCha20-Poly1305 using a fresh random 96-bit nonce per message.
Show a fingerprint (a hash of both public keys) so two users can compare it out-of-band. Without this, the server can swap keys and E2EE means nothing. Store the first key you see per contact and warn if it ever changes (TOFU).

Known limitation: with static-static DH, one leaked key exposes all past messages, because there’s no forward secrecy. That’s fine for v1, and it’s the exact motivation for the later upgrade.

Crates

tokio, axum (with the ws feature), tokio-tungstenite (client), serde, serde_json, x25519-dalek, hkdf, sha2, chacha20poly1305, rand, clap, anyhow.

Milestones (each one is “done when X”)
Echo server. Done when a client connects over WS and the server echoes text back.
Register + lookup. Done when two clients register and lookup bob returns his pubkey. Duplicate usernames are rejected.
Plaintext relay. Done when send bob hi shows up on Bob’s terminal. Use tokio::select! on stdin and the socket in the client.
Add encryption. Done when the server logs show only ciphertext and Bob still reads the message. Verify by printing what the server sees.
Offline mailbox. Done when a message sent while Bob is offline arrives on his next connect.
Fingerprints + TOFU. Done when a changed key triggers a loud warning.
Tests. Done when integration tests drive the server with oneshot or a test WS client, and a crypto round-trip unit test covers tamper detection (flip a bit, expect decryption failure).
Gotchas to expect
Don’t hold the Mutex guard across an .await. This is your first real async-Rust lesson, so use a short scope or tokio::sync::Mutex.
Use an unbounded or bounded mpsc per connection for outgoing frames, and clean up the map entry when the socket drops.
Decide early whether the mailbox stores whole Incoming messages, so offline delivery is just replaying them.
Don’t treat this as production crypto. Composing primitives yourself is fine for learning, but the real app should use a vetted protocol design (X3DH + Double Ratchet, or Noise).
