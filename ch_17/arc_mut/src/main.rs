use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{Duration, sleep};

//shee could have used joinset
//why do we need to use a mutex here? Since at one time only one async task will be
//incrementing the ctr? Is it because sometime, the runtime might run the task in a
//dift thread?

//the tokio::sync::mutex prevents this cos lock is an async method and it doesnt block
//in the tokio version. when one task goes to sleep, the other task just waits on the lock
//in the std mutex but in case of the tokio one, all of the other tasks check the lock once
//and await for the poll on it. (not block the runtime waiting of the task)

#[tokio::main]
async fn main() {
    let mut tasks = Vec::new();
    let ctr = Arc::new(Mutex::new(0));

    //spawn 5 tasks
    for _ in 0..4 {
        let ctr = Arc::clone(&ctr);
        let t = tokio::spawn(async move {
            //each task increments the count once and hands
            //control to the runtime (done 1k times per task)
            for _ in 0..1000 {
                //block here, cos compiler throws send not implemented when i use
                //the drop fn
                {
                    let mut ctr = ctr.lock().await;
                    *ctr += 1;
                }
                task::yield_now().await;
            }
        });

        tasks.push(t);
    }

    //creating another block for scoping issues in shadow var names
    {
        let ctr = Arc::clone(&ctr);
        let t = tokio::spawn(async move {
            for _ in 1..1000 {
                {
                    let mut ctr = ctr.lock().await;
                    *ctr += 1;
                }
                sleep(Duration::from_secs(5)).await;
            }
        });

        //t is moved into tasks
        tasks.push(t);
    }
    //the above block locks the ctr cos 1k secs and goes to sleep
    //this means that when the ctrl is given to the runtime and
    //other tasks want to aquire the lock, it creates a deadlock
    //situation

    let _ = join_all(tasks).await;

    println!("{:?}", ctr);
}
