fn main() {
    let s = String::from("hello world");
    let num = word_ctr(&s);
    println!("{num}");
}

//can call the split_whitespace method and count the number of elements
//in the array.
//handles single white space between the words
fn word_ctr(s: &str) -> u32 {
    let s = s.trim();
    println!("{s}");

    if s.len() == 0 {
        return 0;
    }

    let mut counter = 1;
    for c in s.chars() {
        if c == ' ' {
            counter += 1
        }
    }
    return counter;
}
