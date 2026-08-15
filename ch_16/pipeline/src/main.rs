use std::{sync::mpsc, thread};

fn main() {
    let mut handles = Vec::new();
    let mut res = Vec::new();

    let (txg, rxg) = mpsc::channel();
    let generator = thread::spawn(move || {
        for i in 1..21 {
            txg.send(i).unwrap();
        }
    });

    handles.push(generator);

    //transformer uses rxg to receive the nums
    let (txt, rxt) = mpsc::channel();
    let transformer = thread::spawn(move || {
        //receive the num from generator and
        //square and send each number down
        for num in rxg {
            txt.send(num * num).unwrap();
        }
    });

    handles.push(transformer);

    //do we have to wait until both those threads are completed
    //before collecting into vec ? No because the receiver will be
    //active until atleast one sender is alive (damn how do they do that)

    for num in rxt {
        res.push(num);
    }

    println!("res {res:?}");

    //wait for the threads to finish | just in case
    //though it wouldn't matter in this case? ah no it does
    //matter ig maybe this should be before square ?
    for handle in handles {
        handle.join().unwrap();
    }
}
