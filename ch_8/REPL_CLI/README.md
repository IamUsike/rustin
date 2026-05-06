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
