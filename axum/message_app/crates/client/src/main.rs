use futures_util::{SinkExt, StreamExt};
use std::ops::ControlFlow;
use std::time::Instant;
use tokio::task::JoinSet;
use tokio_tungstenite::tungstenite::Utf8Bytes;

use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::{CloseFrame, Message, frame::coding::CloseCode},
};

const N_CLIENTS: usize = 2;
const SERVER: &str = "ws://localhost:3000/ws";

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    //spawn several clients that will talk concurrently to the server.
    let mut clients = (0..N_CLIENTS).map(spawn_client).collect::<JoinSet<_>>();

    //wait for all our clients to exit
    while clients.join_next().await.is_some() {}

    let end_time = Instant::now();

    //total time should be the same no matter how many clients we spawn
    println!(
        "Total time taken {:#?} with {N_CLIENTS} concurrent clients, should be about 6.45 seconds.",
        end_time - start_time
    );
}

//creates a client, quietly exits on failure
async fn spawn_client(who: usize) {
    let ws_stream = match connect_async(SERVER).await {
        Ok((stream, resonse)) => {
            println!("Handshake for client {who} has been completedd");
            //this will be the http response, same as srv, this is the last moment
            //http headers can be acceessed
            println!("server resonse was {resonse:?}");
            stream
        }
        Err(e) => {
            println!("Websoket for the client {who} failed with {e}!");
            return;
        }
    };

    let (mut sender, mut receiver) = ws_stream.split();

    //we can ping the server for start
    sender
        .send(Message::Ping(axum::body::Bytes::from_static(
            b"hello, server!",
        )))
        .await
        .expect("cant send");

    //spawn an async sender to push more messages to the server
    let mut send_task = tokio::spawn(async move {
        for i in 1..30 {
            if sender
                //in any ws error break the loop
                .send(Message::Text(format!("Message num {i}....").into()))
                .await
                .is_err()
            {
                return;
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        // When we are done we may want our client to close connection cleanly.
        println!("Sending close to {who}...");
        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: Utf8Bytes::from_static("Goodbye"),
            })))
            .await
        {
            println!("Could not send Close due to {e:?}, probably it is ok?");
        };
    });

    //receiver just prints whatever it gets
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // print message and break if instructed to do so
            if process_message(msg, who).is_break() {
                break;
            }
        }
    });

    //wait for either task to finish and kill the other task
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }
}

/// Function to handle messages we get (with a slight twist that Frame variant is visible
/// since we are working with the underlying tungstenite library directly without axum here).
fn process_message(msg: Message, who: usize) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {who} got str: {t:?}");
        }
        Message::Binary(d) => {
            println!(">>> {who} got {} bytes: {d:?}", d.len());
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {who} got close with code {} and reason `{}`",
                    cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow got close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {who} got pong with {v:?}");
        }
        // Just as with axum server, the underlying tungstenite websocket library
        // will handle Ping for you automagically by replying with Pong and copying the
        // v according to spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} got ping with {v:?}");
        }

        Message::Frame(_) => {
            unreachable!("This is never supposed to happen")
        }
    }
    ControlFlow::Continue(())
}
