# Design walkthrough and gaps

The design has one core idea: **the server is a post office that can't read the letters.** It knows who exists and passes sealed messages along. All secrets stay on the clients.

## Who holds what

| | Server (memory only) | Client (files on disk) |
| --- | --- | --- |
| Holds | Username to `UserEntry` map | Own private key |
| | Each user's public key | Known contacts' public keys (TOFU) |
| | Live `Sender`, if the user is online | |
| | Mailbox of undelivered ciphertext | |
| Never holds | Private keys, plaintext | Other users' private keys |

A server restart wipes all accounts and mailboxes. That is accepted for v1.

**Identity:** a user is a username plus a public key. There is no password and no HTTP login. Everything after the WS upgrade is JSON messages on one socket.

## One connection on the server

This is usually the confusing part. Each client gets two tasks and one channel:

```
             ┌─ reader task ──► handles ClientMsg
Bob's socket ┤
             └─ writer task ◄── mpsc channel ◄── Sender (stored in Bob's UserEntry)
```

**Why the channel exists:** Alice's reader task must deliver to Bob, but Bob's writer half belongs to Bob's writer task. So Alice's task finds Bob's `Sender` in the map and pushes the message. Bob's writer task then does the actual socket write.

## Flows

### Register

1. Bob connects, and the server creates his two tasks and channel.
2. Bob sends `Register{username, pubkey}`.
3. The server locks the map. If the name is free, it stores the key and Bob's `Sender`.
4. The server replies `Registered` or `Taken`.
5. Bob's connection remembers "I am bob". This is how `from` is filled later.

### Alice messages Bob, both online

1. Alice types `send bob hi`.
2. Alice's client doesn't have Bob's key, so it holds the message and sends `Lookup{bob}`.
3. The server replies `PubKey{bob, key}`. Alice's client checks it against her TOFU file.
4. Alice's client combines her private key and Bob's public key through X25519, then HKDF, to get a symmetric key.
5. The client encrypts with a fresh nonce and sends `Send{to: bob, nonce, ciphertext}`.
6. The server sets `from` to alice and pushes `Incoming` into Bob's channel. Bob's writer task sends it.
7. Bob's client gets Alice's public key (from his TOFU file, or by lookup) and derives the **same** symmetric key from his private key and her public key.
8. Bob's client decrypts and prints the message.

The server sees only ciphertext in steps 5–6. Milestones 1–3 skip the crypto steps.

### Bob offline

- At step 6, Bob has no live `Sender`, so the server pushes `Incoming` onto his mailbox.
- When Bob reconnects, the server replays the mailbox through his channel. Replay uses the same delivery path as live messages.

### Client loop

`select!` waits on stdin and the socket at once. It can never pause to wait for a reply, because incoming messages must keep flowing. That's why steps 2–3 need a "waiting for key" holding area.

## Why fingerprints exist

The server hands out public keys, so it can lie. It could give Alice its own key instead of Bob's, then decrypt, read, re-encrypt, and forward. Encryption alone doesn't stop this.

- **Fingerprint:** Alice and Bob compare a hash of both keys over the phone or in person. A mismatch means someone swapped a key.
- **TOFU:** the client saves the first key seen for each contact and warns if it changes later.

## Still undecided

- How returning Bob proves he is Bob. This is the login gap.
- What replies cover `Send` to unknown users and malformed JSON.
- How reconnect avoids races with the old connection's cleanup.

## Check your understanding

1. Why can't Alice's reader task write directly to Bob's socket?
2. The server stores only public keys. How do Alice and Bob end up with the same symmetric key without ever sending it?
3. Who fills in `from`, and what breaks if the client supplies it?
4. Why does the mailbox still protect privacy when the server stores messages for days?
