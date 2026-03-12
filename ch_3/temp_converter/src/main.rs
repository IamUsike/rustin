use std::io;

fn main() {
    loop {
        println!(
            "1. To convert from fahrenheit to celcius enter '1'\n2. To convert from celsius to fahrenheit enter '2'"
        );

        let mut input: String = String::new();
        //In full, the let mut guess = String::new(); line has created a mutable variable that is currently bound to a new, empty instance of a String. Whew!

        //read_line appends to the string without overwriting it.
        //read_line returns a `Result` value which is an enum.
        //Result contains variants `Ok` and `Err`.
        //expect is a method defined on Result type. If the instance of Result returns
        //err, expect will cause program to panic.
        //If `Ok` is returned, expect will return the value "Ok" is holding.
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        //trim end is to remove the trailing and leading whitespaces and new line.
        let num_input: i32 = input.trim().parse().expect("Invalid number entered");

        println!("Enter the value to be converted");
        let mut value: String = String::new();

        io::stdin()
            .read_line(&mut value)
            .expect("Failed to read Line");

        let value_input: f32 = value.trim().parse().expect("not a number");

        if num_input == 1 {
            let fah = value_input * 9.0 / 5.0 + 32.0;
            println!("{value_input}°C = {fah}°F");
        } else if num_input == 2 {
            let cel = (value_input - 32.0) * (5.0 / 9.0);
            println!("{value_input}°F = {cel}°C");
            println!("The celsius value for {value_input}F is {cel}");
        }
    }
}
