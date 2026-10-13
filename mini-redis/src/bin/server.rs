use bytes::Bytes;
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

//we need the db to preserve strate across connections
type Db = Arc<Mutex<HashMap<String, Bytes>>>;

/*
* Rule to remember: inside one async function, code is always sequential, just like normal code. Concurrency only happens across separate tasks (tokio::spawn) or with combinators like join! and select!
*/
#[tokio::main]
async fn main() {
    //bind the listener to the ip address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("listening");

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        //second item contains the IP and port of the new connection
        let (socket, _) = listener.accept().await.unwrap();

        //clone the handle of the hashmap
        let db = db.clone();

        //a new task is spawned for each inbound socket.
        //the socket is moved to the new task and processed there.
        println!("accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};

    //the connection lets us read/write redis frames instead
    //of byte streams;
    let mut connection = Connection::new(socket);

    //use read_frame to receive a command from the connection
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    // `Frame::Bulk` expects data to be of type `Bytes`. This
                    // type will be covered later in the tutorial. For now,
                    // `&Vec<u8>` is converted to `Bytes` using `into()`.
                    println!("got val....");
                    Frame::Bulk(value.clone())
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
