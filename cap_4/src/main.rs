use std::collections::HashSet;

fn main() {
    let str = String::from("Hi, my name is.\nHi, my name is.\nWhat?\nchika chika slim shady");
    println!("{str}");

    let (sen, words, chars) = display_stats(&str);

    println!(
        "Number of sentences: {sen}\nNumber of words: {words}\nNumber of unique chars: {chars}"
    );
}

fn display_stats(s: &str) -> (usize, usize, usize) {
    let sentences: usize = s.split('.').count();

    let words: usize = s.split_whitespace().count();

    let mut uniq_chars: HashSet<char> = Default::default();
    let s: String = s.to_lowercase().split_whitespace().collect();

    for c in s.chars() {
        uniq_chars.insert(c);
    }

    (sentences, words, uniq_chars.len())
}
