struct Fibonacci {
    curr: u32,
    next: u32,
}

impl Fibonacci {
    //crete a new series
    fn new(curr: u32, next: u32) -> Self {
        Fibonacci { curr, next }
    }
}

impl Iterator for Fibonacci {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let temp = self.curr;
        self.curr = self.next;
        self.next = temp + self.curr;
        Some(temp)
    }
}

fn main() {
    let mut fib = Fibonacci::new(0, 1);

    for _ in 1..10 {
        if let Some(i) = fib.next() {
            println!("{i}");
        }
    }
    // assert_eq!(fib.next(), Some(1));
}
