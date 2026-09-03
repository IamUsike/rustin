use tokio::sync::mpsc;
use tokio::task::yield_now;

#[tokio::main]
async fn main() {
    let (txa, mut rxa) = mpsc::channel(32);
    let (txb, mut rxb) = mpsc::channel(32);

    let task_a = tokio::spawn(async move {
        //move is required here cos, we need txa to go out
        //of scope when the for loop ends. Else the program
        //wont terminate. (see why below)
        let loo = async move {
            for _ in 0..5 {
                txa.send(String::from("ping")).await.unwrap();
                //yield control to the runtime,from here, the runtime will run
                //the rxb recv or poll task b. Encountering either is fine cos
                //each of them await in the bg until polled
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

    //await for a ping; once received, send pong
    let task_b = tokio::spawn(async move {
        while let Some(message) = rxa.recv().await {
            println!("{message} received");
            txb.send(String::from("pong")).await.unwrap();
            // yield_now().await;
        }
    });

    let _ = tokio::join!(task_a, task_b);
}

/*
1. for rxb to drop, txb has to drop.
2. for txb to drop, rxa has to drop.
3. for rxa to drop, txa has to drop
4. if we move txa into the new async block scope,
it'll get dropped when the scope ends (RAII lol)
*/
