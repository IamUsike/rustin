## Pipeline: generate → transform → collect

Build a 3-stage pipeline using 2 channels. Stage 1 (generator thread): sends numbers 1–20 down channel A. Stage 2 (transformer thread): receives from channel A, squares each number, sends down channel B. Stage 3 (main thread): receives from channel B and collects into a Vec. Print the final Vec. This is the Unix pipe | | pattern in Rust.

> Cements: channels compose into pipelines, each stage is independent, backpressure is automatic

---

future me: read the main.rs first always before reading the below

:cry why does gpt understand my code better than me. smh

- generator does :

```
send 1
send 2
send 3
...
```

- transformer can simulatenously do:

```
receive 1 → square → send 1
receive 2 → square → send 4
receive 3 → square → send 9
...
```

- and main can actually do:

```
receive 1 → Vec
receive 4 → Vec
receive 9 → Vec
...
```

- so there are concurrent stages:

```
Generator:    1 ── 2 ── 3 ── 4 ── 5 ──>
                ↓
Transformer:      1² ── 2² ── 3² ── 4² ──>
                         ↓
Main:                  collect ── collect ──>
```

- update:
  `for num in rxt` continues until the channel is disconnected — i.e. until all Senders for that channel have been dropped.

the final join matters? (not really sure).
but it makes the main wait until both those threads are terminated.

in the program, by the time this executes:

```
for num in rxt {
  res.push(num)
}
```

- the transformer must have finished sending, cos the receiver loop only ends when `txt`
  is dropped.
- and the transformer can't finish until the generator's `rxg` loop ends.

So practically, by the time you finish collecting:

```
generator ── finished
     ↓
transformer ── finished
     ↓
channel B closed
     ↓
main's rxt loop ends
```

Therefore the joins will likely return immediately.

> But keeping the joins is still correct, because you're explicitly guaranteeing that the worker threads have terminated before main exits.(sure bro 😀)
