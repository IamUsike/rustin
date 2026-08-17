use std::sync::{Arc, Mutex};
use std::thread;

/*

fn main() {
    let aa = Arc::new(Mutex::new(0));
    let bb = Arc::new(Mutex::new(0));

    let mut handles = Vec::new();

    let a = Arc::clone(&aa);
    let b = Arc::clone(&bb);
    let thread_a = thread::spawn(move || {
        println!("Thread_a aquiring lock on a");
        let _q = a.lock().unwrap();
        println!("thread a aquired lock on a, proceeding to lock b");
        let _r = b.lock().unwrap();
        println!("Thread a aquired lock b");
    });

    let a = Arc::clone(&aa);
    let b = Arc::clone(&bb);
    let thread_b = thread::spawn(move || {
        println!("Thread b aquiring lock on b");
        let _r = b.lock().unwrap();
        println!("Thread b aquired lock on b, proceeding to lock a");
        let _q = a.lock().unwrap();
        println!("Thread b aquired lock on b");
    });

    handles.push(thread_a);
    handles.push(thread_b);

    for handle in handles {
        handle.join().unwrap();
    }
}

//OUTPUT

//Thread_a aquiring lock on a
//Thread b aquiring lock on b
//thread a aquired lock on a, proceeding to lock b
// Thread b aquired lock on b, proceeding to lock a

output explanation:
- Main thread starts -> Thead a starts -> aquires lock on a ; Thread b starts -> aquires lock on b;
- thread a tries to lock a but thread b has the lock. So thread a is waiting.
- thread b tries to lock a but thread a has the lock. So thread b is waiting.
- both are perpetually waiting for the locks to be released.
*/

///////////////////////////////////////////////

/* an excerpt from oracle docs regd deadlocks

* An example of another kind of deadlock is when two threads, thread 1 and thread 2, each acquires a mutex lock, A and B, respectively. Suppose that thread 1 tries to acquire mutex lock B and thread 2 tries to acquire mutex lock A. Thread 1 cannot proceed and it is blocked waiting for mutex lock B. Thread 2 cannot proceed and it is blocked waiting for mutex lock A. Nothing can change, so this is a permanent blocking of the threads, and a deadlock.

This kind of deadlock is avoided by establishing an order in which locks are acquired (a lock hierarchy). When all threads always acquire locks in the specified order, this deadlock is avoided.
*/

/*

fn main() {
    let aa = Arc::new(Mutex::new(0));
    let bb = Arc::new(Mutex::new(0));

    let mut handles = Vec::new();

    let a = Arc::clone(&aa);
    let b = Arc::clone(&bb);
    let thread_a = thread::spawn(move || {
        println!("Thread_a aquiring lock on a");
        let _q = a.lock().unwrap();
        println!("thread_a aquired lock on a, proceeding to lock b");
        let _r = b.lock().unwrap();
        println!("Thread_a aquired lock b");
    });

    let a = Arc::clone(&aa);
    let b = Arc::clone(&bb);
    let thread_b = thread::spawn(move || {
        println!("Thread_b aquiring lock on a");
        let _q = a.lock().unwrap();
        println!("thread_b aquired lock on a, proceeding to lock b");
        let _r = b.lock().unwrap();
        println!("Thread_b aquired lock b");
    });

    handles.push(thread_a);
    handles.push(thread_b);

    for handle in handles {
        handle.join().unwrap();
    }
}

//////////output: (will be different each run: even the previous deadlock vala code would also give
//difft outputs for each run)
// Thread_b aquiring lock on a
// thread_b aquired lock on a, proceeding to lock b
// Thread_b aquired lock b
// Thread_a aquiring lock on a
// thread_a aquired lock on a, proceeding to lock b
// Thread_a aquired lock b

// Output run 2
// Thread_a aquiring lock on a
// Thread_b aquiring lock on a
// thread_a aquired lock on a, proceeding to lock b
// Thread_a aquired lock b
// thread_b aquired lock on a, proceeding to lock b
// Thread_b aquired lock b

Output explanation: Should be self explanatory from the oracle quote and o/p anways
- main thred starts and one of the threads aquire lock on a. (assuming thread b)
- thread a keeps waiting for the lock on a
- thread b finishes its job with a and then releases the lock and proceeds to
aquire lock on b.
- if thread a wants to get a lock on b, it waits for thread b to release the lock
- once thread b releases the lock
- so no deadlocks
 */

/* next para from the same oracle doc
Adhering to a strict order of lock acquisition is not always optimal. When thread 2 has many assumptions about the state of the module while holding mutex lock B, giving up mutex lock B to acquire mutex lock A and then reacquiring mutex lock B in order would cause it to discard its assumptions and reevaluate the state of the module.

The blocking synchronization primitives usually have variants that attempt to get a lock and fail if they cannot, such as mutex_trylock(). This allows threads to violate the lock hierarchy when there is no contention. When there is contention, the held locks must usually be discarded and the locks reacquired in order.


*/

/*
fn main() {
    let aa = Arc::new(Mutex::new(0));
    let bb = Arc::new(Mutex::new(0));

    let mut handles = Vec::new();

    let a = Arc::clone(&aa);
    let b = Arc::clone(&bb);
    let thread_a = thread::spawn(move || {
        println!("Thread_a aquiring lock on a");
        let _q = a.try_lock().unwrap();
        println!("thread a aquired lock on a, proceeding to lock b");
        let _r = b.try_lock().unwrap();
        println!("Thread a aquired lock b");
    });

    let a = Arc::clone(&aa);
    let b = Arc::clone(&bb);
    let thread_b = thread::spawn(move || {
        println!("Thread b aquiring lock on b");
        let _r = b.try_lock().unwrap();
        println!("Thread b aquired lock on b, proceeding to lock a");
        let _q = a.try_lock().unwrap();
        println!("Thread b aquired lock on b");
    });

    handles.push(thread_a);
    handles.push(thread_b);

    for handle in handles {
        handle.join().unwrap();
    }
}

*/

/*
///// OUTPUT
- sometimes we get the output and other times the code panics.
- (if I ran the first deadlock wala code multiple times, It would either
run properly(if one thread finishes exection first )  or get into a deadlock state)
- the above code panics because try_lock sends Err when the lock couldn't
be aquired
- tho i'm not really sure in which cases try lock would be useful (readme)

- for ex: whenever the therads dont panic the output will invariably be one of these
2 (cos one thread needs to finish execting first)

-> CASE 1
hread b aquiring lock on b
Thread b aquired lock on b, proceeding to lock a
Thread b aquired lock on b
Thread_a aquiring lock on a
thread a aquired lock on a, proceeding to lock b
Thread a aquired lock b

-> CASE 2
Thread_a aquiring lock on a
thread a aquired lock on a, proceeding to lock b
Thread a aquired lock b
Thread b aquiring lock on b
Thread b aquired lock on b, proceeding to lock a
Thread b aquired lock on b
*/

//sheet I forgot to loop
//remember the locks are dropped automatically(if not done manually), when
//the lock goes out of scope. RAII

fn main() {
    let lock_a = Arc::new(Mutex::new(0));
    let lock_b = Arc::new(Mutex::new(0));

    let mut handles = Vec::new();

    let a = Arc::clone(&lock_a);
    let b = Arc::clone(&lock_b);

    let t1 = thread::spawn(move || {
        match a.try_lock() {
            Ok(guard) => {
                println!("t1 acquired lock on a");
                drop(guard);
            }
            Err(_) => loop {
                match a.try_lock() {
                    Ok(guard) => {
                        println!("t1 acquired a after loop");
                        drop(guard);
                        break;
                    }
                    Err(_) => {
                        continue;
                    }
                }
            },
        }

        // match b.try_lock() {
        //     Ok(guard) => {
        //         println!("t1 acquired lock on b");
        //         drop(guard);
        //     }
        //     Err(_) => loop {
        //         println!("t1 err when acquiring b");
        //     },
        // }

        let _q = b.lock().unwrap();
        println!("t2 has lock on b");
    });

    let a = Arc::clone(&lock_a);
    let b = Arc::clone(&lock_b);

    let t2 = thread::spawn(move || {
        // match b.try_lock() {
        //     Ok(guard) => {
        //         println!("t2 acquired lock on b");
        //         drop(guard);
        //     }
        //     Err(_) => loop {
        //         println!("t2 no lock on b");
        //     },
        // }
        //
        // match a.try_lock() {
        //     Ok(guard) => {
        //         println!("t2 acquired lock on a");
        //         drop(guard);
        //     }
        //     Err(_) => loop {
        //         println!("t2 no lock a")
        //     },
        // }

        let _q = b.lock().unwrap();
        println!("t2 has lock on b");

        let _r = a.lock().unwrap();
        println!("t2 has lock on a");
    });

    handles.push(t1);
    handles.push(t2);

    for handle in handles {
        handle.join().unwrap();
    }
}

/*
-> only loop is reqd for a: i'm too lazy to write why but
just trace the flow
*/
