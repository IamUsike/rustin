//I wont write the program with normal i32 cos all the threads need to
//own the value. Rc  and refcell cos ik rc and
//refcell dont implement the send or sync traits so they aren't safe
//to send over threads.

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let ctr = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    //spawn 5 threads
    for _ in 0..5 {
        let ctr = Arc::clone(&ctr);
        let handle = thread::spawn(move || {
            //each thread increments the counter 1k times
            for _ in 0..1000 {
                let mut num = ctr.lock().unwrap();
                *num += 1;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("The final result is: {}", *ctr.lock().unwrap());
}
