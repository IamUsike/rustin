use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};

//lets make it such that each worker sleeps for 0.5 secs and prints:
//worker completed some task

//'static: The closure doesn't contain references that could become invalid while the closure is alive.
type Task = Box<dyn FnOnce() + Send + 'static>;

//instead of storing strings, a real threadpool stores work
struct ThreadPool {
    //contains the current threads
    workers: Vec<JoinHandle<()>>,
    receiver: Arc<Mutex<mpsc::Receiver<Task>>>,
    sender: Option<mpsc::Sender<Task>>,
}

impl ThreadPool {
    fn new(n: usize) -> Self {
        //create a receiver
        //spawn n workers
        let (tx, rx) = mpsc::channel::<Task>();
        let receiver = Arc::new(Mutex::new(rx));

        //handle to store workers
        let mut workers = Vec::new();

        //create n threads
        for i in 0..n {
            //owned refernce of rx (threads need to own that value);
            let rx = Arc::clone(&receiver);
            let worker = thread::spawn(move || {
                loop {
                    //every thread waits for the lock simultaneously
                    match rx.lock().unwrap().recv() {
                        Ok(f) => {
                            println!("thread {i} doing someshi");
                            f();
                        }
                        Err(_) => break,
                    }
                }
            });

            workers.push(worker);
        }

        Self {
            workers,
            receiver,
            sender: Some(tx),
        }
    }

    fn execute(&self, f: Task) {
        if let Some(tx) = self.sender.clone() {
            tx.send(f).unwrap();
        }
    }

    // fn shutdown(self) {
    //     drop(self.sender);
    // }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        while let Some(worker) = self.workers.pop() {
            worker.join().unwrap();
        }
    }
}

fn main() {
    let f = Box::new(|| {
        println!("hello\n");
    }) as Box<dyn FnOnce() + Send + 'static>;

    let tp = ThreadPool::new(3);

    for _ in 0..12 {
        let f = Box::new(f);
        tp.execute(f);
    }

    // tp.execute(f);

    // for worker in tp.workers {
    //     worker.join().unwrap();
    // }
}

/* absolutes
-> fn new : spawns n worker threads
-> All workers need to share the same receiver
-> Ideally, the new method should create workers and
the receiver ?
*
*
*
*
*/
