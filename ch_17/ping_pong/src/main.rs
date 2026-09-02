use tokio::sync::mpsc;
use tokio::task::yield_now;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let (txa, mut rxa) = mpsc::channel(32);
    let (txb, mut rxb) = mpsc::channel(32);

    let task_a = tokio::spawn(async move {
        let loo = async move {
            for _ in 1..5 {
                txa.send(String::from("ping")).await.unwrap();
                //send ping and give control to the runtime, it can either
                //execute the below asyc block or
                yield_now().await;
            }
        };

        let lis_a = async {
            while let Some(message) = rxb.recv().await {
                println!("{message} received");
            }
        };

        tokio::join!(lis_a, loo);
    });

    println!("after task a");

    let task_b = tokio::spawn(async move {
        while let Some(message) = rxa.recv().await {
            println!("{message} received");
            txb.send(String::from("pong")).await.unwrap();
            // yield_now().await;
        }
    });

    let _ = tokio::join!(task_a, task_b);
}
