use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let mut scores = HashMap::new();
    scores.insert(String::from("alice"), 0);
    scores.insert(String::from("bob"), 0);
    scores.insert(String::from("carol"), 0);
    scores.insert(String::from("dave"), 0);

    let scores = Arc::new(Mutex::new(scores));

    let mut handles = Vec::new();
    //alice increments the score by 1, b 2 and so on.

    let score = Arc::clone(&scores);
    let alice = thread::spawn(move || {
        for _ in 0..5 {
            let mut score = score.lock().unwrap();
            //deref coercion
            let cur_score = score.entry(String::from("alice")).or_insert(0);
            *cur_score += 1;
        }
    });

    let score = Arc::clone(&scores);
    let bob = thread::spawn(move || {
        for _ in 0..5 {
            let mut score = score.lock().unwrap();
            //deref coercion
            let cur_score = score.entry(String::from("bob")).or_insert(0);
            *cur_score += 2;
        }
    });

    let score = Arc::clone(&scores);
    let carol = thread::spawn(move || {
        for _ in 0..5 {
            let mut score = score.lock().unwrap();
            //deref coercion
            let cur_score = score.entry(String::from("carol")).or_insert(0);
            *cur_score += 3;
        }
    });

    let score = Arc::clone(&scores);
    let dave = thread::spawn(move || {
        for _ in 0..5 {
            let mut score = score.lock().unwrap();
            //deref coercion
            let cur_score = score.entry(String::from("dave")).or_insert(0);
            *cur_score += 4;
        }
    });

    handles.push(alice);
    handles.push(bob);
    handles.push(carol);
    handles.push(dave);

    for handle in handles {
        handle.join().unwrap();
    }

    println!("final score is: {scores:?}");
}
