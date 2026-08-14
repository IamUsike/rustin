use std::sync::mpsc;
use std::thread;

//create 2 channels main(ping) -> thread and thread -> main
fn main() {
    let (tx, rx) = mpsc::channel();
    let (tx1, rx1) = mpsc::channel();

    let val = String::from("Ping");
    tx.send(val).unwrap();

    //move tx1 and rx into the thread
    let handle = thread::spawn(move || {
        let received = rx.recv().unwrap();

        println!("Got {received}");

        // let tx1 = tx.clone(); // this wont work cos a receiver can receive messages from its sender
        let val = String::from("pong");
        tx1.send(val).unwrap();
    });

    //receive message from tx1
    let received: String = rx1.recv().unwrap();
    println!("Got {received}");

    handle.join().unwrap();
}
