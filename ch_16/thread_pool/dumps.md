## Condvar

- imagine a producer thread and a consumer thread.

The producer puts work into a queue:

```
Producer                     Consumer
   |                            |
   |                            |
   |---- puts task ------------>|
   |                            |
   |                            | processes task
```

-> what if the consumer checks the queue and it's empty?
A bad soln would be:

```rust
loop {
  if !queue.is_empty() {
    //consume
  }
}
```

**this is busy waiting**

The consumer is continuously using the cpu:

```
check queue
check queue
check queue
check queue
check queue
...
```

Instead we could do:

```
Consumer:
    "Is the queue empty?"
          |
        YES
          |
    go to sleep 😴
          |
          |
Producer adds task
          |
          v
    wake consumer
          |
          v
    consumer processes task
```

- `Condvar` is for this case

### Basic Structure

- A condition var is normally paired with a `Mutex`:

```
use std::sync::{Mutex, Condvar};

let pair = (Mutex::new(false), Condvar::new());
```

```
Mutex (protects the data)
  |
  v
[ shared state ]

Condvar (lets threads wait for that state)
  |
  v
"wake me when the state changes"
```

-> A Simple example
Suppose one thread waits until another thread sets `ready=true`

```rust
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
  // bool reps the conditiion
    let pair = Arc::new((Mutex::new(false), Condvar::new()));

    let pair_clone = Arc::clone(&pair);

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));

        let (lock, cvar) = &*pair_clone;

    //aquire the mutex
        let mut ready = lock.lock().unwrap();

        *ready = true;

    //Wake up one thread currently waiting on this condition variable.
        cvar.notify_one();
    });

    let (lock, cvar) = &*pair;

    let mut ready = lock.lock().unwrap();

    while !*ready {
        ready = cvar.wait(ready).unwrap();
    }

    println!("Ready!");
}
```

when we do:

```rust
ready = cvar.wait(ready).unwrap();
```

the condition variable:

1. Atomically releases the mutex
2. Puts the thread to sleep
3. Waits for notification
4. Wakes up
5. Re-acquires the mutex
6. Returns the mutex guard

Conceptually:

```
Before wait():

Thread
  |
  v
Mutex LOCKED
  |
ready = false


cvar.wait(ready)
       |
       +---- unlock mutex
       |
       +---- sleep 😴
```

the other thread does:

```rust
*ready = true;
cvar.notify_one()
```

The waiting thread wakes:

```
wake up
   |
   v
re-acquire mutex
   |
   v
check ready
   |
   v
true
   |
   v
continue
```

##### Why does `wait()` take the MutexGuard

```rust
cvar.wait(ready)
```

instead of:

```rust
cvar.wait()
```

- **Because condvar needs to coordinate the mutex and the waiting ops**

you give it:

```rust
MutexGuard<bool>
```

and it effectively does:

```
                  Mutex
                    |
                    v
             ┌─────────────┐
             │ ready=false │
             └─────────────┘
                    |
                    |
              cvar.wait()
                    |
             release mutex
                    |
                  sleep
```

when it wakes:

```
wake
 |
 v
re-acquire mutex
 |
 v
return MutexGuard
```

so

```rust
ready = cvar.wait(ready).unwrap();
```

returns another `MutexGuard`

#### Why `While`, and not `if`?

- on first thought, this might seem like a viable approach:

```rust
if !*ready {
    ready = cvar.wait(ready).unwrap();
}
```

- but the standard pattern is:

```rust
while !*ready {
    ready = cvar.wait(ready).unwrap();
}
```

**Because being notified doesn't necessarily mean the condition is true.**

```
Thread wakes up
      |
      v
re-acquires mutex
      |
      v
check condition again
      |
      +---- false ---> sleep again
      |
      +---- true ----> continue
```

There can also be **Spurious wakeups**
So the rule is:

> A Condvar notification means "something may have changed", not "your condition is definitely true."

#### The three things to remember:

```
Mutex
  ↓
protects shared state

Condvar::wait()
  ↓
releases mutex + sleeps
  ↓
wakes
  ↓
reacquires mutex
  ↓
returns

while condition
  ↓
checks whether we should ACTUALLY continue
```

And:

```
notify_one()
    ↓
wake ONE waiter
    ↓
which waiter?
    ↓
NOT GUARANTEED
```

> The really important insight is: the Condvar doesn't wait for your condition. It waits for a notification/wakeup; your while loop is responsible for determining whether the condition is actually satisfied.
