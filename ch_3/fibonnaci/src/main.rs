const MAX_FIB: u64 = 12586269025;

fn main() {
    fib_iter(7);
    for i in 0..7 {
        println!("{}", fib_recur(i));
    }
}

fn fib_iter(n: u64) {
    let mut cur: u64 = 1;
    let mut prev: u64 = 0;
    let mut temp: u64;

    println!("Iterative method\n1");

    for _ in 0..n - 1 {
        temp = cur;
        cur += prev;
        prev = temp;
        if cur >= MAX_FIB {
            println!("Element exceeds limit");
            break;
        }
        println!("{cur}");
    }
}

fn fib_recur(n: u64) -> u64 {
    if n <= 1 {
        return n;
    } else {
        return fib_recur(n - 1) + fib_recur(n - 2);
    }
}

//okay man could've used vectors and tuples and theres some
//weird ass way to add max fib in recursio apparently.
