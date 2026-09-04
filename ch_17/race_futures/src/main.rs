/* select! - race futures against each other
- write 2 async tasks: one completes after 200ms(fast task) and the other completes
  after 2000ms("slow task")
- use tokio::select! to select whichever completes first and cancel the other
- Then build a practical version: A "fetch with timeout" pattern - select!
  between your async work and a sleep(timeout). If sleep wins, return an Err("timeout")
*/
use tokio::time::{Duration, sleep};
#[tokio::main]
async fn main() {
    //I feel creating another async fn instead of spawning a
    //tokio task would be better cos the exec(polling) of that fn
    //starts when it encounters tokio::select() unlike spawn
    //which immediately starts running the task in the bg
    let task1 = tokio::spawn(async {
        println!("starting task1");
        sleep(Duration::from_millis(5000)).await;
        String::from("Task successfully completed")
    });
    let sl_time = 100;
    match fetch(task1, sl_time).await {
        //check why the message is Result<> instead of T::Output
        Ok(message) => println!("Succeeded with {:?}!", message),
        Err(message) => {
            println!("Failed with 'Message'");
        }
    }
}

//First param is a generic that implements the future trait
//tokio::spawn returns JoinHandle that implements the Future
//trait. so should suffice. I hope
async fn fetch<T: Future>(task: T, sl_time: u64) -> Result<T::Output, String> {
    tokio::select! {
        val = task => {
            println!("task completed before timeout");
            Ok(val)
        }
        _ = sleep(Duration::from_millis(sl_time)) => {
            println!("timeouutt");
            Err(String::from("timeout"))
        }
    }
}

/* Things to verify
1. we have to invoke fns using () when using select
2. task here, already starts, we check the status
3. do we not have to poll the async fn using await(within select!)
   -> we dont, cos select handles the polling and shit
*/

//TWO TASKS RACE USING SELECT
// #[tokio::main]
// async fn main(){
//     let task1 = tokio::spawn( async {
//         sleep(Duration::from_millis(200)).await;
//         String::from("fast task")
//     });
//     let task2 = tokio::spawn(async {
//         sleep(Duration::from_millis(2000)).await;
//         String::from("slow task")
//     });
//     tokio::select! {
//         res = task1 => {
//             println!("task1");
//             // println!("{:?}",res);
//         }
//         res = task2 => {
//             println!("task2");
//             // println!("{:?}",res);
//         }
//     }
// }
