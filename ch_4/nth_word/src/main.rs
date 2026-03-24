fn main() {
    let s = String::from("hello wassup bish how ya doin");
    let fw = first_word(&s);
    println!("{fw}");

    let n: usize = 3;
    let word = nth_word(&s, n);
    println!("{n}th word: {word}");
}

fn first_word(s: &str) -> &str {
    let ws_index = s.trim().find(' ');

    //convert option<usize> to <usize>
    let int_index = match ws_index {
        None => 0,
        Some(i) => i,
    };

    if int_index == 0 { &s } else { &s[0..int_index] }
    // &s[0..ws_index]
}

fn nth_word(s: &str, n: usize) -> &str {
    let word = s.split_whitespace().nth(n);
    let word = match word {
        None => "",
        Some(i) => i,
    };
    println!("{:?}", word);
    word
}
