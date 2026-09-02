/* tokio::spawn vs tokio::join!
- spawn 3 tasks with tokio::spawn; each sleeps a diff amount and prints when done.
- Collect the JoinHandles and await them.
- Do the same thing with join!
- comment expl.
- DROP a JoinHandle without awaiting it, what happens?
*/

use tokio::time::{Duration, sleep};

/*
//collect and await
#[tokio::main]
async fn main() {
    let mut tasks = Vec::new();

    let handle1 = tokio::spawn(async {
        sleep(Duration::from_millis(500)).await;
        println!("handle1 completed after 500 ms");
    });

    tasks.push(handle1);

    let handle2 = tokio::spawn(async {
        sleep(Duration::from_millis(1000)).await;
        println!("handle2 completed after 1000ms");
    });

    tasks.push(handle2);

    let handle3 = tokio::spawn(async {
        sleep(Duration::from_millis(1500)).await;
        println!("handle3 completed after 1500ms");
    });

    tasks.push(handle3);

    //this is sequential?
    //each task starts executing right after it is spawned
    //this for loop is essential for the main thread to wait for the tasks to finish
    //if we do not push any task(handle) to the handles vec, the task still executes
    //but we cant guarantee its completion cos main might finish executing before and
    //the runtime will get dropped (program terminate)
    //for eg: even if we do not await handle1 or handle2, we can still guarantee their
    //completion cos handle3 runs for 1500ms
    for task in tasks {
        task.await.unwrap();
    }
}

*/

#[tokio::main]
async fn main() {
    let handle1 = tokio::spawn(async {
        sleep(Duration::from_millis(500)).await;
        println!("handle1 completed after 500ms");
    });

    let handle2 = tokio::spawn(async {
        sleep(Duration::from_millis(1000)).await;
        println!("handle2 completed after 1000ms");
    });

    let handle3 = tokio::spawn(async {
        sleep(Duration::from_millis(1500)).await;
        println!("handle3 completed after 1500ms");
    });

    //awaiting a JoinHandle produces a Result<T, tokio::task::JoinError>.
    //So tokio::join! is producing a tuple like:
    //(Result<..., JoinError>, Result<..., JoinError>, Result<..., JoinError>)

    //This silences the warning but still discards errors (use sparingly):
    //Ideally need to error match statement or ? on each Result
    let _ = tokio::join!(handle1, handle2, handle3);
}

/*
- the key diff between storing the handles and awaiting is that the tasks themselves will
start execution when they are spawned but the await is blocking(sequential), ie: We await
the next task only when one task is finished.
But when we do tokio::join! the await is concurrent ie: if one task is blocking the runtime
moves to the next task
*/
