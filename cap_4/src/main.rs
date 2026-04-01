use std::collections::HashSet;

fn main() {
    let str = String::from("Hi, my name is.\nHi, my name is.\nWhat?\nchika chika slim shady");

    let (sen, words, chars) = display_stats(&str);

    println!(
        "Number of sentences: {sen}\nNumber of words: {words}\nNumber of unique chars: {chars}"
    );

    let longest_word: &str = longest_word(&str);
    println!("The longest word is: \"{longest_word}\" ");

    let replace_word: String = replace_word(&str, "slim", "fat");
    println!("{replace_word}");
}

fn display_stats(s: &str) -> (usize, usize, usize) {
    //let sentences = s.matches(|c| c == '.' || c == '?' || c == '!').count();
    let sentences: usize = s.split('.').count();

    let words: usize = s.split_whitespace().count();

    let mut uniq_chars: HashSet<char> = Default::default();

    // for c in s.to_lowercase().chars() {
    //     if !c.is_whitespace() {
    //         uniq_chars.insert(c);
    //     }
    // }
    //unnecessary cloning
    let s: String = s.to_lowercase().split_whitespace().collect();

    for c in s.chars() {
        uniq_chars.insert(c);
    }

    (sentences, words, uniq_chars.len())
}

//could create a hashet and push to that in case we want
//all the longest words
fn longest_word(s: &str) -> &str {
    let words: Vec<&str> = s.split_whitespace().collect();
    let mut len: usize = 0;
    let mut longest_word: &str = "";
    for word in words {
        if word.len() > len {
            len = word.len();
            longest_word = word;
        }
    }

    longest_word
}

//this is better, doesnt have unnecessary vec allocation
// fn longest_word(s: &str) -> &str {
//     let mut longest = "";
//
//     for word in s.split_whitespace() {
//         if word.len() > longest.len() {
//             longest = word;
//         }
//     }
//
//     longest
// }

fn replace_word(s: &str, w: &str, r: &str) -> String {
    let s = s.replace(w, r);
    s
}
