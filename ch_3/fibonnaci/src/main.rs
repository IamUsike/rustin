const MAX_FIB: u64 = 12586269025;

fn main() {
    fib_iter(7);
}

fn fib_iter(n: u64) {
    let mut cur: u64 = 1;
    let mut prev: u64 = 0;
    let mut temp: u64;

    println!("1");

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

fn fib_recur(n: u64) {}
