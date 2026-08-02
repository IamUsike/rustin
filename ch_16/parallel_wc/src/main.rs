use std::{fmt::Debug, sync::mpsc, thread};

fn main() {
    let s1 = String::from("Marshall Mathers");
    let s2 = String::from("kobbiee mainooo");
    let s3 = String::from("Rezeeeeeeee");
    let s4 = String::from("miku kawaiii");
    let s5 = String::from("me coax");

    let mut v = Vec::new();
    v.push(&s1);
    v.push(&s2);
    v.push(&s3);
    v.push(&s4);
    v.push(&s5);

    #[derive(Debug)]
    struct StrInfo<'a> {
        str: &'a str,
        len: usize,
    }

    // spawn 1 thread per string
    let (tx, rx) = mpsc::channel();

    for i in 0..5 {
        let tx = tx.clone();
        let v = v.clone();
        thread::spawn(move || {
            let info = StrInfo {
                str: v.get(i).unwrap(),
                len: v.get(i).unwrap().len(),
            };
            tx.send("Length of v.get({i}): {v.get(i).len()}").unwrap();
        });
    }

    //This drop the tx from the main thread.
    //If the tx from the main thread isnt dropped manually,
    //the receiver(rx) waits for the tx indefinetly(4 tx's from
    //the spawned thread will be read but the one on the main thread
    //still remains and it gets dropped automatically when it goes out
    //of scope which in this case would be after the main function)
    drop(tx);
    for received in rx {
        println!("Got: {received}");
    }
}
