fn main() {
    let s = String::from("helloooo    wprld");
    let num = word_ctr(&s);
    println!("wrd ctr: {num}");

    let num = wc(&s);
    println!("wc: {num}");

    let num = wc_gpt(&s);
    println!("wc_gpt: {num}");
}

//can call the split_whitespace method and count the number of elements
//in the array.
//handles single white space between the words
fn word_ctr(s: &str) -> u32 {
    let s = s.trim();

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

fn wc(s: &str) -> u32 {
    let s = s.trim().split_whitespace();
    let mut ctr = 0;
    for _ in s {
        ctr += 1;
    }

    return ctr;
}

fn wc_gpt(s: &str) -> usize {
    s.split_whitespace().count()
}
