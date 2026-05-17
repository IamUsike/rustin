fn main() {
    let a = "a,b,c
1,2,3
x,y,z ";

    let str = String::from(a);

    let op = parse_csv(&str);

    match op {
        Ok(_) => println!("Good"),
        Err(error) => println!("{}", error),
    };
}

//i'll parse the csv and store in a 2-D Array ?

//type 1
// Split at new line and store in a vector
// split at commmas and store in a 2d array
// if len() any array doesnt match the first return err
// this will take O(m+n)
fn parse_csv(input: &str) -> Result<(), String> {
    let v_nl: Vec<&str> = input.split("\n").collect();

    let f_len: Vec<&str> = v_nl[0].split(',').collect();
    let f_len = f_len.len();

    for v in v_nl {
        let v_tmp: Vec<&str> = v.split(",").collect();

        if v_tmp.len() != f_len {
            return Err(String::from("Invalid Column Lengths"));
        };
    }

    Ok(())
}
