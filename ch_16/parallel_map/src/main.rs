// Write a function parallel_map(data: Vec<i32>, f: fn(i32) -> i32) -> Vec<i32> that applies f to each element using one thread per element. The output Vec must be in the same order as the input — not whatever order threads finish. Think about how to preserve order before writing any code.
// Cements: threads don't finish in order, you must account for that explicitly

use std::thread;

fn parallel_map(data: Vec<i32>, f: fn(i32) -> i32) -> Vec<i32> {
    let mut handles = Vec::new();
    let mut result = Vec::new();

    for d in data {
        let handle = thread::spawn(move || f(d));

        //each thread is pushed in order to the vec
        //this is particulary helpful cos later we can just unwrap them (Result)
        //and then get the value inside the thread.
        //this helps in preserving the order. (they might finish in whatever order)
        handles.push(handle); // preserve association with input position
    }

    for handle in handles {
        result.push(handle.join().unwrap()); // collect in handle order
    }

    result
}

fn rnd(x: i32) -> i32 {
    x * 2
}

fn main() {
    let v = vec![1, 2, 3, 4, 5];
    let res = parallel_map(v, rnd);
    println!("res: {:?}", res);
}

/* side note - can also use closures for the fn
*

fn calculate(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

let square = |x| x * x;
println!("{}", calculate(square, 5)); // 25

*/
