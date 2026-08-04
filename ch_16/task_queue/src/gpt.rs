use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

//////////////////////////////////////////////////////////////////
// A Task
//
// Instead of storing Strings, a real thread pool stores WORK.
//
// FnOnce() means:
//     "A callable object (closure/function) that consumes itself
//      and returns nothing."
//
// Send means:
//     It is safe to move this task to another thread.
//
// 'static means:
//     The task owns everything it captures.
//     It cannot borrow local variables that might disappear.
//
//////////////////////////////////////////////////////////////////

type Task = Box<dyn FnOnce() + Send + 'static>;

//////////////////////////////////////////////////////////////////
// Everything shared between workers.
//
// Arc
//  |
//  +--> Shared
//          |
//          +--> Mutex<State>
//          |
//          +--> Condvar (wake workers)
//          |
//          +--> Condvar (wait_all)
//
//////////////////////////////////////////////////////////////////

struct Shared {
    // Protects all mutable shared state.
    state: Mutex<State>,

    // Workers sleep on this until new work arrives.
    work_available: Condvar,

    // wait_all() sleeps on this until every task is complete.
    all_done: Condvar,
}

//////////////////////////////////////////////////////////////////
// The actual mutable shared data.
//
// queue
//     Pending work.
//
// active
//     Number of workers CURRENTLY executing a task.
//
// shutdown
//     When true, workers should exit.
//
//////////////////////////////////////////////////////////////////

struct State {
    queue: VecDeque<Task>,

    active: usize,

    shutdown: bool,
}

//////////////////////////////////////////////////////////////////
// ThreadPool
//
// workers
//     Owns worker threads.
//
// shared
//     Shared state used by every worker.
//
//////////////////////////////////////////////////////////////////

struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    shared: Arc<Shared>,
}

impl ThreadPool {
    //////////////////////////////////////////////////////////////
    // Create a new thread pool.
    //
    // num_workers = number of worker threads.
    //////////////////////////////////////////////////////////////

    pub fn new(num_workers: usize) -> Self {
        //////////////////////////////////////////////////////////
        // Create the shared state.
        //////////////////////////////////////////////////////////

        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                queue: VecDeque::new(),

                // Initially no worker is executing.
                active: 0,

                // Pool is alive.
                shutdown: false,
            }),

            work_available: Condvar::new(),

            all_done: Condvar::new(),
        });

        let mut workers = Vec::new();

        //////////////////////////////////////////////////////////
        // Spawn N worker threads.
        //////////////////////////////////////////////////////////

        for _ in 0..num_workers {
            // Every worker shares ownership of Shared.
            let shared = Arc::clone(&shared);

            let handle = thread::spawn(move || {
                ////////////////////////////////////////////////////
                // Workers never stop.
                //
                // They keep:
                //
                // wait
                // get task
                // execute
                // repeat
                ////////////////////////////////////////////////////

                loop {
                    //////////////////////////////////////////////////
                    // Acquire one task.
                    //////////////////////////////////////////////////

                    let task = {
                        // Lock shared state.
                        let mut state = shared.state.lock().unwrap();

                        //////////////////////////////////////////////////
                        // No work?
                        //
                        // Sleep until submit() wakes us.
                        //////////////////////////////////////////////////

                        while state.queue.is_empty() && !state.shutdown {
                            state = shared.work_available.wait(state).unwrap();
                        }

                        //////////////////////////////////////////////////
                        // Pool shutting down?
                        //
                        // Exit thread.
                        //////////////////////////////////////////////////

                        if state.shutdown && state.queue.is_empty() {
                            return;
                        }

                        //////////////////////////////////////////////////
                        // This worker is now busy.
                        //////////////////////////////////////////////////

                        state.active += 1;

                        //////////////////////////////////////////////////
                        // Take ONE task.
                        //////////////////////////////////////////////////

                        state.queue.pop_front().unwrap()
                    };

                    //////////////////////////////////////////////////////
                    // IMPORTANT
                    //
                    // Mutex is NO LONGER held here.
                    //
                    // We never execute user code while holding a mutex.
                    //
                    // Otherwise every worker would block behind us.
                    //////////////////////////////////////////////////////

                    task();

                    //////////////////////////////////////////////////////
                    // Task finished.
                    //////////////////////////////////////////////////////

                    let mut state = shared.state.lock().unwrap();

                    state.active -= 1;

                    //////////////////////////////////////////////////////
                    // Queue empty?
                    //
                    // Nobody executing?
                    //
                    // Then wait_all() may continue.
                    //////////////////////////////////////////////////////

                    if state.queue.is_empty() && state.active == 0 {
                        shared.all_done.notify_all();
                    }
                }
            });

            workers.push(handle);
        }

        Self { workers, shared }
    }

    //////////////////////////////////////////////////////////////
    // Submit a task.
    //
    // Does NOT create a thread.
    //
    // Just pushes work into the queue.
    //////////////////////////////////////////////////////////////

    pub fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut state = self.shared.state.lock().unwrap();

        state.queue.push_back(Box::new(task));

        //////////////////////////////////////////////////////////
        // Wake ONE sleeping worker.
        //////////////////////////////////////////////////////////

        self.shared.work_available.notify_one();
    }

    //////////////////////////////////////////////////////////////
    // Wait until every submitted task finishes.
    //////////////////////////////////////////////////////////////

    pub fn wait_all(&self) {
        let mut state = self.shared.state.lock().unwrap();

        //////////////////////////////////////////////////////////
        // Two conditions must BOTH be true.
        //
        // 1. Queue empty.
        // 2. No worker currently executing.
        //////////////////////////////////////////////////////////

        while !(state.queue.is_empty() && state.active == 0) {
            state = self.shared.all_done.wait(state).unwrap();
        }
    }
}

//////////////////////////////////////////////////////////////////
// Drop
//
// Called automatically when ThreadPool disappears.
//
//////////////////////////////////////////////////////////////////

impl Drop for ThreadPool {
    fn drop(&mut self) {
        //////////////////////////////////////////////////////////
        // Tell every worker to stop.
        //////////////////////////////////////////////////////////

        {
            let mut state = self.shared.state.lock().unwrap();
            state.shutdown = true;
        }

        //////////////////////////////////////////////////////////
        // Wake everyone.
        //
        // Sleeping workers need to notice shutdown.
        //////////////////////////////////////////////////////////

        self.shared.work_available.notify_all();

        //////////////////////////////////////////////////////////
        // Wait for every worker thread to terminate.
        //////////////////////////////////////////////////////////

        while let Some(worker) = self.workers.pop() {
            worker.join().unwrap();
        }
    }
}

//////////////////////////////////////////////////////////////////
// Demo
//////////////////////////////////////////////////////////////////

fn main() {
    // Create 3 worker threads.
    let pool = ThreadPool::new(3);

    // Submit 10 tasks.
    //
    // Notice:
    // submit() DOES NOT spawn threads.
    //
    // Only 3 worker threads exist.
    //
    // They keep taking tasks from the queue.
    //
    for i in 0..10 {
        pool.submit(move || {
            println!("Task {} executed on thread {:?}", i, thread::current().id());
        });
    }

    // Wait until everything finishes.
    pool.wait_all();

    println!("All tasks completed.");
}
