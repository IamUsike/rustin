**Anagram groups**

Given a `Vec<&str>` of words, group them into anagram families (`Vec<Vec>`).
E.g. `['eat','tea','tan','ate','nat','bat']` → `[['eat','tea','ate'],['tan','nat'],['bat']]`.

_Hint:_ Sort each word’s chars as key for the `HashMap`. Use `entry().or_insert_with(Vec::new).push(...)`.

---

- `or_insert`: Always evaluates value immediately, even if the key already exists.
- `or_insert_with`: Lazy evaluation — the closure runs only if the key is missing.
