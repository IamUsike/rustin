# minichat

A learning project: an [axum](https://github.com/tokio-rs/axum) WebSocket relay and a CLI client that exchange end-to-end encrypted 1:1 messages. The server routes and stores ciphertext but never sees plaintext.

> **Status:** work in progress. The commands below describe the planned layout and will not work until the matching milestone is built.
>
> **Not production crypto.** This composes primitives by hand for learning and has no forward secrecy. Do not use it to protect real messages.

## How it works

```
client A ──ws──► server (routes by username, stores ciphertext) ──ws──► client B
```

- Each client generates an X25519 keypair and fetches the other user's public key from the server.
- Clients derive a shared key with X25519 and HKDF-SHA256, then encrypt with ChaCha20-Poly1305.
- Messages for offline users wait in an in-memory mailbox until the next connect.
- Clients compare key fingerprints out-of-band and warn if a contact's key changes (TOFU).

## Scope

**In:** register, 1:1 messages, E2EE, offline delivery, key fingerprint check.

**Out:** groups, media, persistence, Tor, forward secrecy.

## Layout

| Crate | Purpose |
| --- | --- |
| `crates/proto` | Wire messages shared by server and client |
| `crates/server` | axum WebSocket relay |
| `crates/client` | CLI client, crypto, and local key storage |

## Usage

```sh
# Start the relay
cargo run -p server

# Start a client (use a separate data dir for each user)
cargo run -p client -- --data-dir /tmp/alice

# Run checks
cargo test --workspace
cargo clippy --workspace
```

## Milestones

- [ ] 1. Echo server
- [ ] 2. Register and lookup
- [ ] 3. Plaintext relay
- [ ] 4. Encryption
- [ ] 5. Offline mailbox
- [ ] 6. Fingerprints and TOFU
- [ ] 7. Tests

The build guide is in [CLAUDE.md](CLAUDE.md).

## Known limitations

- Static-static DH: one leaked key exposes all past messages.
- The server can replay stored ciphertext undetected.
- Mailboxes live in memory and are lost when the server restarts.
