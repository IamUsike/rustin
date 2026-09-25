# Deliberately break lifetimes — 5 ways

Write 5 programs that each cause a specific lifetime error. For each: write the broken version, read the error carefully, fix it, explain in a comment what the borrow checker caught and why it was right to reject the code. The 5 errors: (1) return a reference to a local variable. (2) use a reference after the owned value was moved. (3) have two mutable references to the same data. (4) use a struct containing a reference after the referee was dropped. (5) call a function that takes &mut self while holding a &self reference.

> What this cements: Every lifetime error the compiler produces is preventing real memory unsafety. Understanding what bug each error prevents makes the errors feel helpful instead of hostile.
