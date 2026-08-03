use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

type Task = Box<dyn FnOnce() + Send + 'static>;

struct Shared {
    state: Mutex<State>,
    work_available: Condvar,
    all_done: Condvar,
}

struct State {
    queue: VecDeque<Task>,
    active: usize,
    shutdown: bool,
}

struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    shared: Arc<Shared>,
}

impl ThreadPool {
    pub fn new(num_workers: usize) -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                queue: VecDeque::new(),
                active: 0,
                shutdown: false,
            }),
            work_available: Condvar::new(),
            all_done: Condvar::new(),
        });

        let mut workers = Vec::new();

        for _ in 0..num_workers {
            let shared = Arc::clone(&shared);

            workers.push(thread::spawn(move || {
                loop {
                    let task = {
                        let mut state = shared.state.lock().unwrap();

                        while state.queue.is_empty() && !state.shutdown {
                            state = shared.work_available.wait(state).unwrap();
                        }

                        if state.shutdown && state.queue.is_empty() {
                            return;
                        }

                        state.active += 1;
                        state.queue.pop_front().unwrap()
                    };

                    task();

                    let mut state = shared.state.lock().unwrap();
                    state.active -= 1;

                    if state.queue.is_empty() && state.active == 0 {
                        shared.all_done.notify_all();
                    }
                }
            }));
        }

        Self { workers, shared }
    }

    pub fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut state = self.shared.state.lock().unwrap();
        state.queue.push_back(Box::new(task));
        self.shared.work_available.notify_one();
    }

    pub fn wait_all(&self) {
        let mut state = self.shared.state.lock().unwrap();

        while !(state.queue.is_empty() && state.active == 0) {
            state = self.shared.all_done.wait(state).unwrap();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        {
            let mut state = self.shared.state.lock().unwrap();
            state.shutdown = true;
        }

        self.shared.work_available.notify_all();

        while let Some(worker) = self.workers.pop() {
            worker.join().unwrap();
        }
    }
}

fn main() {
    let pool = ThreadPool::new(3);

    for i in 0..10 {
        pool.submit(move || {
            println!("Task {} executed on thread {:?}", i, thread::current().id());
        });
    }

    pool.wait_all();

    println!("All tasks completed.");
}
