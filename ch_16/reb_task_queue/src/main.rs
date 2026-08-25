use core::task;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

type Task = Box<dyn FnOnce() + Send + 'static>;

struct TaskQueue {
    //join handle is a generic type
    workers: Vec<JoinHandle<()>>,
    queue: Mutex<VecDeque<Task>>,
    tasks_pending: Mutex<u32>,
    //the same condvar will be shared by multiple threads
    wake: Arc<Condvar>,
    //worker to keep awake
    keep_awake: Arc<Mutex<bool>>,
}

impl TaskQueue {
    //spawn n workers and wait
    fn new(n: u32) -> Self {
        let mut workers = Vec::new();
        //just initialize it ?
        let tasks_pending = Mutex::new(0);
        let queue = Mutex::new(VecDeque::<Task>::new());
        let wake = Arc::new(Condvar::new());
        let keep_awake = Arc::new(Mutex::new(false));

        for _ in 0..n {
            let wake = Arc::clone(&wake);
            let keep_awake = Arc::clone(&keep_awake);

            let worker = thread::spawn(move || {
                let mut guard = keep_awake.lock().unwrap();

                while !*guard {
                    //if we dont re-assign guard here, upar vaala guard will be moved
                    //into the loop and the guard that's checked in the while loop
                    //will be invalid after the first iteration
                    guard = wake.wait(guard).unwrap();

                    //code to be executed after getting the lock should be written
                    //the while loop cos of spurious wakes.
                }
            });

            workers.push(worker);
        }

        Self {
            workers,
            queue,
            tasks_pending,
            wake,
            keep_awake,
        }
    }

    fn submit<T>(&self, task: T)
    where
        T: FnOnce() + Send + 'static,
    {
    }
}

fn main() {
    println!("Hello, world!");
}

/* absolutes
-> TaskQueue - holds N worker threads and a shared Mutex for pending tasks
-> We need the threads to wait for a job on condvar
    - Can this be done by using deque.len ? or do I need a separate var?
-> threads will be waiting for tasks(loop), when a task is submitted, we
notify_one (wake any thread)
*
*
*
*
*
*
*
*/
