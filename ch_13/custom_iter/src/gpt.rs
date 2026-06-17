struct Fibonacci {
    // Current Fibonacci number
    curr: u64,

    // Next Fibonacci number
    next: u64,
}

impl Fibonacci {
    // Create a new Fibonacci sequence starting at 0, 1
    fn new() -> Self {
        Self { curr: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    // Every call to next() returns a u64
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        // Save the current value because that's what
        // we want to return.
        let current = self.curr;

        // Advance the sequence:
        //
        // Before:
        // curr = 3
        // next = 5
        //
        // After:
        // curr = 5
        // next = 8

        self.curr = self.next;
        self.next = current + self.curr;

        // Since this iterator is infinite,
        // we always return Some(...)
        Some(current)
    }
}

fn main() {
    let sum: u64 = Fibonacci::new()
        .take(20) // first 20 Fibonacci numbers
        .filter(|n| n % 2 == 0) // keep only even ones
        .sum(); // add them together

    println!("Sum = {}", sum);
}
