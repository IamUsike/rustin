use std::sync::{Arc, mpsc};
use std::thread;

fn main() {
    // These Strings are OWNED by main initially.
    let s1 = String::from("hello");
    let s2 = String::from("kobbiee mainooo");
    let s3 = String::from("Rezeeeeeeee");
    let s4 = String::from("miku kawaiii");
    let s5 = String::from("me coax");

    // Instead of storing references (&String), store the Strings themselves.
    // The Vec now OWNS all the Strings.
    //
    // We wrap the Vec in an Arc because multiple threads need to read from it.
    // Arc gives multiple owners of the same data in a thread-safe manner.
    let v = Arc::new(vec![s1, s2, s3, s4, s5]);

    #[derive(Debug)]
    struct StrInfo<'a> {
        // Borrow a string slice from one of the Strings inside the Vec.
        str: &'a str,

        // Store its length.
        len: usize,
    }

    // Create a channel.
    // tx -> Sender
    // rx -> Receiver
    let (tx, rx) = mpsc::channel();

    // Spawn one thread for every string.
    for i in 0..5 {
        // Each thread needs its own Sender.
        // clone() DOES NOT create another channel.
        // It creates another handle to the SAME channel.
        let tx = tx.clone();

        // Clone the Arc.
        //
        // This DOES NOT clone the Vec<String>.
        // It only increments Arc's atomic reference count.
        //
        // Every thread still points to the same Vec.
        let v = Arc::clone(&v);

        thread::spawn(move || {
            // move transfers ownership of:
            // 1. tx
            // 2. the Arc clone
            // 3. i (copied because usize implements Copy)
            //
            // After this, the thread owns everything it needs.

            let info = StrInfo {
                // v.get(i) returns Option<&String>.
                // unwrap() gives us &String.
                //
                // &String automatically coerces to &str.
                str: v.get(i).unwrap(),

                len: v.get(i).unwrap().len(),
            };

            // Send the struct to the main thread.
            tx.send(info).unwrap();

            // tx is dropped here automatically.
        });

        // Arc clone is also dropped when the thread exits.
    }

    // Drop the ORIGINAL sender.
    //
    // If we don't do this, rx will wait forever because
    // one Sender would still exist in main.
    drop(tx);

    // Receive messages until ALL senders are dropped.
    //
    // This loop internally keeps calling recv().
    //
    // It exits automatically when:
    // - every Sender has been dropped, AND
    // - the channel becomes empty.
    for received in rx {
        println!("{received:?}");
    }
}
