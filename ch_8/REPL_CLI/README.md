**hard · capstone**

**In-memory key-value store CLI**

Build a CLI REPL that supports:

- SET key value
- GET key
- DELETE key
- LIST (all keys)
- HISTORY (last 10 commands)

Use `HashMap` for storage and `Vec` for history. Parse commands from stdin in a loop.

**Hint:**
Use a loop `{ let mut input = String::new(); stdin.read_line(&mut input); ... }` pattern. Split on whitespace to parse commands.

---

ps: REPL : Read – Eval – Print – Loop

---

## Notes

> Note that `std::env::args` will panic if any argument contains invalid Unicode. If your program needs to accept arguments containing invalid Unicode, use std::env::args_os instead. That function returns an iterator that produces OsString values instead of String values. We’ve chosen to use std::env::args here for simplicity because OsString values differ per platform and are more complex to work with than String values.
