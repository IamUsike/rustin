**Ownership Rules • Borrowing • References • Slices**

**Challenge 10: String Ownership Transfer [Easy]**

Write a function `takes_ownership(s: String)` that prints the string. Call it in `main`. Try to use the string after — observe the compile error. Fix by (a) cloning, and (b) rewriting the function to take a reference instead.

**Hints:** String vs `&str` • `clone()` makes a deep copy

---

A reference is like a pointer in that it’s an address we can follow to access the data stored at that address; that data is owned by some other variable. Unlike a pointer, a reference is guaranteed to point to a valid value of a particular type for the life of that reference.

##### Raw pointer mindset (C/C++)

Can point anywhere
Can be null
Can dangle
No guarantees
You manually ensure safety

#### Rust reference (&T)

A reference is a pointer + rules enforced by the compiler

It guarantees:

1. Always valid

- No dangling references

2. Never null
3. Correct type
4. Borrow rules enforced

- Either:
  - many immutable references (&T)
  - OR one mutable reference (&mut T)

5. Lifetime checked

- Cannot outlive the data it points to
