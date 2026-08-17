## Deadlock — deliberately cause it, then fix it

Write a program with two mutexes (lock_a and lock_b) and two threads. Thread 1 locks A then B. Thread 2 locks B then A. Run it and watch it deadlock (it will hang forever). Then fix it two ways: (1) consistent lock ordering (both threads lock A then B), (2) try_lock with a retry loop. Understanding deadlocks is as important as avoiding them.

> Cements: what deadlock actually is, lock ordering as prevention, try_lock as escape hatch

---

```rust
use std::sync::{Arc, Mutex};
use std::thread;

/*
===============================================================
TRY_LOCK + RETRY LOOP
===============================================================

The goal:

Thread 1 wants: A -> B
Thread 2 wants: B -> A

This is intentionally an inconsistent lock ordering.

Normally, this can deadlock:

    T1: holds A -> waits for B
    T2: holds B -> waits for A

With try_lock(), we DON'T block waiting for the second lock.

Instead:

    1. Acquire the first lock using try_lock()
    2. Try to acquire the second lock
    3. If the second lock is unavailable:
           - release the first lock
           - retry from the beginning
    4. Only enter the critical section once BOTH locks
       have been successfully acquired.


===============================================================
*/

fn main() {
    let lock_a = Arc::new(Mutex::new(0));
    let lock_b = Arc::new(Mutex::new(0));

    let mut handles = Vec::new();

    // =========================================================
    // THREAD 1
    //
    // Wants: A -> B
    // =========================================================

    let a = Arc::clone(&lock_a);
    let b = Arc::clone(&lock_b);

    let t1 = thread::spawn(move || {
        loop {
            // -------------------------------------------------
            // Step 1: Try to acquire A.
            //
            // Unlike lock(), try_lock() DOES NOT BLOCK.
            // If A is unavailable, we simply retry.
            // -------------------------------------------------

            let guard_a = match a.try_lock() {
                Ok(guard) => {
                    println!("T1 acquired A");
                    guard
                }

                Err(_) => {
                    // Couldn't get A.
                    // Start another attempt.
                    continue;
                }
            };

            // -------------------------------------------------
            // Step 2: We have A.
            //
            // Now try to acquire B.
            // -------------------------------------------------

            let guard_b = match b.try_lock() {
                Ok(guard) => {
                    println!("T1 acquired B");
                    guard
                }

                Err(_) => {
                    // -------------------------------------------------
                    // IMPORTANT:
                    //
                    // We already hold A, but B is unavailable.
                    //
                    // DO NOT keep holding A while retrying!
                    //
                    // Release A and retry from the beginning.
                    // -------------------------------------------------

                    drop(guard_a);

                    println!("T1 couldn't acquire B -> releasing A");

                    continue;
                }
            };

            // -------------------------------------------------
            // We only reach here if we own BOTH A and B.
            //
            // Safe critical section.
            // -------------------------------------------------

            println!("T1 acquired BOTH A and B");

            // Do work with A and B here...

            drop(guard_b);
            drop(guard_a);

            println!("T1 released A and B");

            // Successfully completed the operation.
            break;
        }
    });

    // =========================================================
    // THREAD 2
    //
    // Intentionally uses the opposite order: B -> A
    //
    // This would normally be dangerous.
    // try_lock + retry prevents us from getting stuck.
    // =========================================================

    let a = Arc::clone(&lock_a);
    let b = Arc::clone(&lock_b);

    let t2 = thread::spawn(move || {
        loop {
            // -------------------------------------------------
            // Step 1: Try to acquire B.
            // -------------------------------------------------

            let guard_b = match b.try_lock() {
                Ok(guard) => {
                    println!("T2 acquired B");
                    guard
                }

                Err(_) => {
                    // Couldn't get B.
                    continue;
                }
            };

            // -------------------------------------------------
            // Step 2: We have B.
            //
            // Now try to acquire A.
            // -------------------------------------------------

            let guard_a = match a.try_lock() {
                Ok(guard) => {
                    println!("T2 acquired A");
                    guard
                }

                Err(_) => {
                    // -------------------------------------------------
                    // We have B but couldn't get A.
                    //
                    // Release B before retrying.
                    // Otherwise we could recreate the deadlock:
                    //
                    // T1 holds A -> wants B
                    // T2 holds B -> wants A
                    //
                    // -------------------------------------------------

                    drop(guard_b);

                    println!("T2 couldn't acquire A -> releasing B");

                    continue;
                }
            };

            // -------------------------------------------------
            // We now own BOTH locks.
            // -------------------------------------------------

            println!("T2 acquired BOTH B and A");

            // Do work with A and B here...

            drop(guard_a);
            drop(guard_b);

            println!("T2 released A and B");

            break;
        }
    });

    handles.push(t1);
    handles.push(t2);

    for handle in handles {
        handle.join().unwrap();
    }
}


/*
================================================================
CONTRAST WITH MY ORIGINAL CODE
================================================================

MY ORIGINAL CODE:

    match a.try_lock() {
        Ok(guard) => {
            println!("T1 acquired A");
            drop(guard);             // <-- RELEASE A HERE
        }

        Err(_) => {
            loop {
                match a.try_lock() {
                    Ok(guard) => {
                        println!("T1 acquired A");
                        drop(guard);
                        break;
                    }

                    Err(_) => continue,
                }
            }
        }
    }

    let _q = b.lock().unwrap();


The problem is that A is released BEFORE we try to acquire B.

So the flow is:

    try_lock(A)
        |
        +-- got A
        |
        +-- release A
        |
        +-- lock(B)


Therefore we're NEVER doing:

    hold A
       |
       +----> try B


So there is no opportunity for the classic deadlock situation
to occur between these two operations.


================================================================
THE IMPORTANT DIFFERENCE
================================================================

MY ORIGINAL:

    try A
      |
      v
    release A
      |
      v
    lock B


PROPER try_lock SOLUTION:

    try A
      |
      v
    HOLD A
      |
      v
    try B
      |
      +---- success ---> HOLD A + B ---> critical section
      |
      +---- failure
               |
               v
          release A
               |
               v
             retry


The retry applies to the ACQUISITION OF BOTH LOCKS,
not just the first lock.


================================================================
WHY release the first lock?
================================================================

Suppose:

    T1 gets A
    T2 gets B

Then:

    T1: try B -> fails
    T2: try A -> fails

If they keep their existing locks:

    T1: holds A, waits for B
    T2: holds B, waits for A

DEADLOCK.

Instead:

    T1: try B -> fails -> release A -> retry
    T2: try A -> fails -> release B -> retry

Nobody holds one lock while permanently waiting for the other.


================================================================
THREE WAYS WE LEARNED
================================================================

1. DELIBERATE DEADLOCK

       T1: A -> B
       T2: B -> A

       A <--- T1
       ^
       |
       T2
       |
       B <--- T2
       ^
       |
       T1

       Circular wait -> DEADLOCK


2. LOCK ORDERING

       T1: A -> B
       T2: A -> B

       Everyone follows the same hierarchy.

       No circular wait -> NO DEADLOCK


3. try_lock + RETRY

       T1: A -> try B
       T2: B -> try A

       If second lock unavailable:
           release first lock
           retry

       Don't wait while holding the first lock.

================================================================
CRUX
================================================================

lock()
    = "Wait until I get the lock."

try_lock()
    = "Try NOW; don't wait."

try_lock() + retry
    = "Try to acquire all required locks.
       If I can't get them all, release what I have
       and try again."

The important part isn't merely using try_lock().
The important part is:

    FAILED SECOND LOCK
          |
          v
    RELEASE FIRST LOCK
          |
          v
        RETRY
*/
```
