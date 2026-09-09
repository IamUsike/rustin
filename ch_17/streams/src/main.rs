// use std::fmt::format;

use tokio::sync::mpsc;
// use tokio_stream::StreamExt;
use tokio::time::{Duration, sleep};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

//p2
async fn publisher(tx: mpsc::Sender<String>) {
    tokio::spawn(async move {
        for i in 1..8 {
            let _ = tx.send(format!("message {}", i)).await;
            sleep(Duration::from_millis(300)).await;
            println!("publisher finished sleeping for 300ms, incoming new message")
        }
    });
}

async fn receiver(rx: mpsc::Receiver<String>) {
    let mut resp = ReceiverStream::new(rx);

    while let Some(res) = resp.next().await {
        println!("{res}");
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);

    // let _ = tokio::spawn(publisher(tx));
    publisher(tx).await;

    //should the publisher and receiver be sequential? No, because once publisher hits
    //sleep maybe(or tokio::spawn?), ctrl is given to the runtime and then receiver is
    //called.
    receiver(rx).await;

    //main doesn't go out of scope cos receiver wont terminate until all the senders are dropped
    //thus we just wait at receiver.await
}

//part 1
// #[tokio::main]
// async fn main() {
//     //convert it into a stream
//     let stream = tokio_stream::iter(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 16, 20]);
//
//     let mut sq = stream.filter(|x| *x % 2 == 0).map(|x| x * x).take(5);
//
//     while let Some(v) = sq.next().await {
//         println!("{v}");
//     }
// }
