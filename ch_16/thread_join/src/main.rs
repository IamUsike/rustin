// spawn 5 threads -> Each thread prints its own id(0-4). In main join all of them
use std::thread;
use std::time::Duration;

// fn spawn_thread()
// store all the handles in a vec and then join them.
// fn main() {
//     let mut threads = Vec::new();
//     for i in 0..5 {
//         let handle = thread::spawn(move || {
//             println!("Thread with id {i}")
//         });
//         // push each thread to the vec array
//         threads.push(handle);
//     }
//     for t in threads {
//         t.join().unwrap()
//     }
// }
// //the current flow would be,
// //each thread is spawned and then the handle is pushed to a new vec
// //the execution of each thread would be random (scheduled by the OS)
// //this is why the number printed is seemingly random
// //we then iterate through the handles and then wait for them to finish
// //(join returns immediately if the thread has finished execution)
// }

// //for the above code make it such that threads sleep for (5-i)*100ms before printing
fn main() {
    let mut threads = Vec::new();
    for i in 0..5 {
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis((5 - i) * 100));
            println!("Thread with id {i}");
        });
        threads.push(handle);
    }

    //for takes ownership of the thread
    for t in threads {
        t.join().unwrap();
    }
}
// //the above snippet will always print 4...0
// //this is because each thread is sleeping for a duration (5-i)*100
// //so thread with id 0 would sleep for 500ms while thread with id 4 would
// //sleep for 100ms.
// //when i=0 is waiting for thread 0 (blocking the main thread)
// //all other threads will simultaneously be executing, (cos multithreaded)
//and they print their id

/*
* One subtle point

You said:

the above snippet will always print 4...0

For the practical purpose of this exercise, yes, that's the expected ordering.

But technically, sleeping for 100/200/300/400/500 ms doesn't mathematically guarantee exact ordering because OS scheduling isn't deterministic. A thread waking from sleep() becomes eligible to run; it doesn't necessarily execute at the exact instant the sleep duration expires.

So the better mental model is:

The sleeps make 4 → 3 → 2 → 1 → 0 the overwhelmingly expected order by giving each thread a progressively earlier wake-up time.

And that's exactly what this exercise is trying to teach: without synchronization, thread execution order is nondeterministic; join() gives you completion synchronization, not execution-order synchronization.*/
