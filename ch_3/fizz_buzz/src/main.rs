fn main() {
    for n in 1..111 {
        let reminders = (n % 3, n % 5, n % 7);

        match reminders {
            (0, 0, 0) => println!("FizzBuzzBazz"),
            (_, 0, 0) => println!("BuzzBazz"),
            (0, _, 0) => println!("FizzBazz"),
            (0, 0, _) => println!("FizzBuzz"),
            (0, _, _) => println!("Fizz"),
            (_, 0, _) => println!("Buzz"),
            (_, _, 0) => println!("Bazz"),
            (_, _, _) => println!("{n}"),
        }
    }
}
