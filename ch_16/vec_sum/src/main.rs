//spawn 4 threads each receives a vec<i32>(diff data per thread) and
//collect 4 return values in main and print the total. You must move
//thread, cannot pass a reference

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let v0 = vec![1, 2, 3];
    let v1 = vec![2, 3, 4];
    let v2 = vec![3, 4, 5];
    let v3 = vec![4, 5, 6];

    //contains references to all the vecs
    let vecs = [v0, v1, v2, v3];

    let mut threads = Vec::new();

    //create a mutex: each thread aquires the lock and then pushes i...
    //need arc here, cos this value would be owned by multiple threads
    //this doesnt need to be mutable ????
    let fin_sum = Arc::new(Mutex::new(Vec::new()));

    for v in vecs {
        //create an owned reference
        //this reference will be move din the for loop and
        //will be automatically dropped when the loop ends
        let fin_sum = Arc::clone(&fin_sum);

        let handle = thread::spawn(move || {
            //iter method produces an iterator over immutable references
            let sum: i32 = v.iter().sum();

            //lock the fin_sum(and then push) (the lock is dropped after the thread stops execution)
            let mut vec = fin_sum.lock().unwrap();
            vec.push(sum);
        });

        threads.push(handle);
    }

    //wait for the threads to finish execution (join immediately returns if the thread has
    //finished execution)
    for t in threads {
        t.join().unwrap();
    }

    println!("Vec: {:?}", fin_sum);

    // let sum: i32 = fin_sum.into_inner().unwrap().iter().sum(); //cannot move out of an `Arc`

    //still throws "cannot move out of an Arc" cos this means
    //Clone the arc handle and then try to move the object inside it out
    // let sum = *fin_sum.clone();

    //but sum consumes the vec inside arc?? Hows this possible
    /*
         Arc<Mutex<Vec<i32>>>
                │
                │ lock()
                ▼
        MutexGuard<Vec<i32>>
                │
                │ Deref
                ▼
             Vec<i32>
                │
                │ iter() (borrows the vector). The iterator contains references to the values
                |  inside the vec. (It doesnt own those int or vec!!!!)
                ▼
        Iterator<Item = &i32>
                │
                │ sum()
                ▼
              i32

    - SUM CONSUMES THE ITERATOR AND NOT THE UNDERLYING VALUE

    ```
    let v = vec![1, 2, 3];

    let sum: i32 = v.iter().sum();

    println!("{:?}", v); // ✅ still exists
    ```

    - iterator was consumed but v wasnt

    - into_iter moves the value
    */
    let sum: i32 = fin_sum.lock().unwrap().iter().sum();
    println!("the final sum is {}", sum);
}
