use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

/*
* Rule to remember: inside one async function, code is always sequential, just like normal code. Concurrency only happens across separate tasks (tokio::spawn) or with combinators like join! and select!
*/
#[tokio::main]
async fn main() {
    //bind the listener to the ip address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        //second item contains the IP and port of the new connection
        let (socket, _) = listener.accept().await.unwrap();
        //a new task is spawned for each inbound socket.
        //the socket is moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    //a hashmap to store the data
    let mut db = HashMap::new();

    //the connection lets us read/write redis frames instead
    //of byte streams;
    let mut connection = Connection::new(socket);

    //use read_frame to receive a command from the connection
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                //value stored in `Vec<u8>`
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    // `Frame::Bulk` expects data to be of type `Bytes`. This
                    // type will be covered later in the tutorial. For now,
                    // `&Vec<u8>` is converted to `Bytes` using `into()`.
                    println!("got val....");
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        //write the connection to the client response
        connection.write_frame(&response).await.unwrap();
    }
}
