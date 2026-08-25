use core::task;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

type Task = Box<dyn FnOnce() + Send + 'static>;

struct TaskQueue {
    //join handle is a generic type
    workers: Vec<JoinHandle<()>>,
    //the actual task
    queue: Arc<Mutex<VecDeque<Task>>>,
    tasks_pending: Arc<Mutex<u32>>,
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
        let tasks_pending = Arc::new(Mutex::new(0));

        //needs to be arc, so every thread can clone it
        let queue = Arc::new(Mutex::new(VecDeque::<Task>::new()));
        let wake = Arc::new(Condvar::new());
        let keep_awake = Arc::new(Mutex::new(false));

        for i in 0..n {
            let wake = Arc::clone(&wake);
            let keep_awake = Arc::clone(&keep_awake);
            let queue = Arc::clone(&queue);
            let tasks_pending = Arc::clone(&tasks_pending);

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

                {
                    //aquire the lock on tasks pending and pop the task
                    let mut queue = queue.lock().unwrap();
                    let task = queue.pop_front();
                    //release the lock on the tasks queue immediately cos when a thread is doing
                    //some task other threads should be able to access the queue
                    drop(queue);

                    match task {
                        Some(t) => {
                            println!("thread {i} executing task");
                            t();
                            //decrement the tasks pending by 1
                            let mut tasks_pending = tasks_pending.lock().unwrap();
                            *tasks_pending -= 1;
                        }
                        None => {
                            println!("dk why")
                        }
                    }

                    //if no tasks left, set the wake mutex to false
                    //no need to actually put it in a new block cos this
                    //is the last part of the loop and the lock gets dropped once
                    //the loop ends
                    let tasks_pending = tasks_pending.lock().unwrap();
                    if *tasks_pending == 0 {
                        let mut keep_awake = keep_awake.lock().unwrap();
                        *keep_awake = false;
                    }
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
        //lock the queue and then add a task to it
        //lock tasks_pending and increment it by 1
        //set keep_awake to true (just set it at every task, even though tasks
        //are pending cos it'll take less compute this way)
        //notify condvar that the value has changed

        //implicilty coerced into box<dyn T>. Look up unsizing coercion
        let task = Box::new(task);
        /*
        Mutex<VecDeque<Task>>
               ↓ lock()
        MutexGuard<VecDeque<Task>>
               ↓ Deref
        VecDeque<Task>
               ↓
        push_back(task)
        */
        self.queue.lock().unwrap().push_back(task);

        //increment tasks pending by 1
        let mut tp = self.tasks_pending.lock().unwrap();
        //method calls and operators use dift coercion rules
        *tp += 1;

        let mut keep_awake = self.keep_awake.lock().unwrap();
        *keep_awake = true;

        //notify the condvar that the value has changed
        self.wake.notify_one();
    }

    //wait for all the threads to finish and then shut them down?
    fn wait_all(&self) {}
}

fn main() {
    let pool = TaskQueue::new(3);
    pool.submit(|| {
        println!("hello");
    });
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
