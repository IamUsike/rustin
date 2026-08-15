use rand::RngExt;
use std::sync::mpsc;
use std::{thread, time::Duration};

//a channel disconnects when all senders or all receivers are dropped
fn main() {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();

    for i in 0..3 {
        //cloned before spawning the thread cos tx would move to the closure
        //if we cloned it inside the thread
        //using `arc` wouldn't be helpful here, cos we need all the threads
        //to access tx together (arc would be coupled with mutex here )
        let tx1 = tx.clone();
        let handle = thread::spawn(move || {
            let mut rng = rand::rng();
            let sl_dur = rng.random_range(1000..2000);
            thread::sleep(Duration::from_millis(sl_dur));

            let val = format!("String {} finished the task in {}ms", i, sl_dur);
            tx1.send(val).unwrap();
        });

        handles.push(handle);
    }

    //drop tx immediately. the other clones send messages
    drop(tx);
    let logger = thread::spawn(move || {
        for r in rx {
            println!("{r}");
        }
    });

    //this'll terminate after one message
    // let re = rx.recv().unwrap();
    // println!("{re}");

    // we dont need this cos r will already wait for all the senders
    // logger.join().unwrap();
}

// drop(tx);
