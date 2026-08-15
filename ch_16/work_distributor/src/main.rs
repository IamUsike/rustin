//we cant clone a receiver in an mpsc channel cos of its designed like that +
//that would lead to race conditions. many senders will just send it downstream
//and receiver will receive them when free(peaceful). Else, the most obvious thing could be
//yk each sender will check a free receiver(among it's receivers) and then send
//it to the free receiver. Would this be enough to avoid race conditions? (obv we'll
//use mutex to lock )

use std::{sync::mpsc, thread};
//main sends job using 1 tx..(wrong. read the later part of ques)
//spawn 3 worker threads.
//jobs are sent by the main thread in a round robin manner

fn main() {
    //three channels for three workers
    let (tx0, rx0) = mpsc::channel();
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    //copy trait (read down)
    // let worker_channels = vec![(tx0, rx0), (tx1, rx1), (tx2, rx2)];
    let worker_senders = [tx0, tx1, tx2];
    let mut workers = Vec::new();

    //results channel
    let (txr, rxr) = mpsc::channel();

    for i in 0..21 {
        //cant to this cos tx doesnt implement copy trait
        // let (sender, _) = worker_channels[3 % i];
        // sender.send(i).unwrap();

        //if you want me to write an explanatory comment for this, STOP.
        //nvm. Keep the ownership with vec itself
        worker_senders[i % 3].send(i).unwrap();
    }

    //lets assume each worker multiplies the number by the worker number
    //eg: w0 would do i*0

    //clone the result channel's sender outside closure cos we dont want
    //the sender to move inside it(closure) and then closure'll own it.
    //later it'll be unusable(ownable)
    let txr0 = txr.clone();
    let w0 = thread::spawn(move || {
        //how gawar can I even be to write this code
        // let (_,  receiver)
        // let _num = rx0.recv().unwrap();

        //receive from the main thread and send it down
        //its always 0 so not using any vars
        for _ in rx0 {
            txr0.send(0).unwrap();
        }
    });

    let txr1 = txr.clone();
    let w1 = thread::spawn(move || {
        // let (_,  receiver) = worker_channels[0];
        // let num = rx1.recv().unwrap();

        for num in rx1 {
            txr1.send(num).unwrap();
        }
    });

    let w2 = thread::spawn(move || {
        // let (_,  receiver)
        // let num = rx2.recv().unwrap();

        for num in rx2 {
            txr.send(num * 2).unwrap();
        }
    });

    workers.push(w0);
    workers.push(w1);
    workers.push(w2);

    //result channel shi
    let mut fin_num = Vec::new();

    //this closure can only be executed once, cos it's returning the moved value back
    let res = thread::spawn(move || {
        // let r = rxr.recv().unwrap();
        for r in rxr {
            println!("received {r}");
            fin_num.push(r);
        }

        println!("code coming");
        fin_num
    });

    //drop all the senders;
    //if we dont do this none of receivers of workers wouldn't be dropped
    //and workers would continue execution and then the senders for the
    //result thread which are present in the worker threads would also exist
    //which will lead to the for loop in the result thread existing forever
    drop(worker_senders);

    //dont need this cos res is now returning the fin_num back
    //pushing to workers vec only (sorry uncle bob) |
    // workers.push(res);

    let fin_num = res.join().unwrap();
    println!("final res {fin_num:?}");

    //wait for the threads to finish. Again doesnt really matter cos all receivers will be there
    //but good habit ig

    for worker in workers {
        worker.join().unwrap();
    }
}

/*
* w1, w2, w3.
* 1. move rx out of each worker to the next worker once it receives a message?
* -> Not possible, since all three threads are running parallely, how would I even tell a thread
* wait for the rx?  doesn't make sense
*
* 2. Arc on rx?
* -> this wouldn't work cos then tx would have to aquire the lock on rx before sending anything.
* this is not how it's unimplemented in the std lib
*
//fuckkkkkkkkkk. i read the question wrong fml. I'll just use 3 channels
*
*
*
*
*
*
*/
