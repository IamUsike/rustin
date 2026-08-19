use std::sync::{Arc, Mutex, mpsc};
use std::{collections::HashMap, thread};

// fn main() {
//     //the global map (to store the final counters)
//     let mut ctr: HashMap<String, usize> = HashMap::new();
//
//     let s1 = String::from("rust is fast and rust is safe");
//     let s2 = String::from("rust is fun and programming is fun");
//     let s3 = String::from("systems programming with rust is powerful");
//     let s4 = String::from("rust makes systems programming fun");
//
//     let strings = [s1, s2, s3, s4];
//
//     let mut handles = Vec::new();
//     let (tx, rx) = mpsc::channel();
//
//     for string in strings {
//         //this part is still in the main thread (so ctr isnt populated)
//         //hence, we can clone ctr directly(deep copy)
//         let mut ctr = ctr.clone();
//         let txi = tx.clone();
//
//         let handle = thread::spawn(move || {
//             freq_ctr(string, &mut ctr);
//             txi.send(ctr).unwrap();
//         });
//
//         handles.push(handle);
//     }
//
//     //drop tx (which isnt used by any of threads)
//     //else receiver wont stop. So, suppose the above for loop is in it's third iteration
//     //and then we drop tx. Then it wont be able to do tx.clone ?? let's see
//     //but drop is after the for loop so...it wont get dropped
//     drop(tx);
//
//     for r in rx {
//         //iterate over the hashmap(by taking ownership)
//         for (string, count) in r {
//             //if an entry exists, give a mutable reference to its value.
//             //else, insert 0 and give me a mutable reference to that
//             let c = ctr.entry(string).or_insert(0);
//
//             //add count by the number of occurrences of the word
//             *c += count;
//         }
//     }
//
//     //wait for all the threads to finish
//     for handle in handles {
//         handle.join().unwrap();
//     }
//
//     println!("Final: {:?}", ctr);
// }
//

//let it take ownership cos we dont need string anymore
fn freq_ctr(s: String, ctr: &mut HashMap<String, usize>) {
    // println!("received thread: {:?}", ctr);

    let words: Vec<&str> = s.split_whitespace().collect();

    // println!("received words: {:?}", words);

    //words is being consumed here...
    for word in words {
        // let word = word.to_string();
        let count = ctr.entry(word.to_string()).or_insert(0);
        *count += 1;
    }

    println!("Map for string '{}' is {:?}", s, ctr);

    //dont need to return anything cos we are mutating the reference directly
}

//arc mutex
fn main() {
    let ctr = Arc::new(Mutex::new(HashMap::<String, usize>::new()));

    let s1 = String::from("rust is fast and rust is safe");
    let s2 = String::from("rust is fun and programming is fun");
    let s3 = String::from("systems programming with rust is powerful");
    let s4 = String::from("rust makes systems programming fun");

    let strings = [s1, s2, s3, s4];
    let mut handles = Vec::new();

    for string in strings {
        //make an atomic rc clone. Cos thread needs to own the values
        let ctr = Arc::clone(&ctr);

        let handle = thread::spawn(move || {
            //lock will wait for the lock to be released
            let mut ctr = ctr.lock().unwrap();
            freq_ctr(string, &mut ctr);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {:?}", ctr);
}

/* When To use channels and when arc
--> Channels
- when we want everything each counter to occur without any block, we can choose mpsc
- Arc has less code (as in we dont have to add one more loop for the counter in the main thread)
*/
