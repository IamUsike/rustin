use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

// 10 print tasks with 3 workers.

// first principles ?
//
// 1. Create a struct that holds worker threads and the task queue
//
// 2. We need to lock the task(mutex) so only one thread can access it.
// So, create a struct `Task` and then we can define it as a mutex in the
// task queue.
//
// 3. When we submit a task
//      - push it to the mutex queue. | there should be one instance of the struct hmm
//      - worker thread aquires the lock -> pop the task -> relase the 
//      lock -> complete the task

//hold the owned string representing the task
struct Task {
    task: String,
}

impl Task {
    pub fn new(task: String) -> Self {
        Self {task}
    }
}
//this struct holds N worker threads
//this also needs to hold the shared mutex for pending tasks.
struct TaskQueue {
    worker_threads: Vec<JoinHandle<()>>,
    task_queue : Arc<Mutex<Vec<Task>>>,
}

//
impl TaskQueue {
//  Vec::new()
//     ↓
// empty Vec<Task>
//
// Mutex::new(Vec::new())
//     ↓
// Mutex protecting that Vec<Task>
//
// Arc::new(Mutex::new(Vec::new()))
//     ↓
// shared ownership of that Mutex<Vec<Task>>

    fn new() -> TaskQueue {
        Self { worker_threads: Vec::new(), task_queue : Arc::new(Mutex::new(Vec::new()))}
    }

    // push the task 
    fn submit_task(&mut self, Task){
        let handle = thread::spawn(move || {
           let task_queue = Arc::clone(&self.task_queue); 
           task_queue

        })
    }
}

fn main() {
    let t1 = Task::new(String::from("Hello from t1"));
    let t2 = Task::new(String::from("Hello from t2"));
    let t3 = Task::new(String::from("Hello from t3"));
    let t4 = Task::new(String::from("Hello from t4"));
    let t5 = Task::new(String::from("Hello from t5"));
   
    let tasks = vec![t1, t2, t3, t4, t5];

    let mut tq = TaskQueue::new();


    for task in tasks {
        tq.submit_task(task);
    }

    //create an instance of the struct so that we can store shi ?
    // Should we create 3 threads in main ? 


}
