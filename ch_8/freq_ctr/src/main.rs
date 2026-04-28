use std::collections::HashMap;

fn main() {
    let s = String::from("hi my name is chika chika slim shady slim slim ");
    let freq = frq_ctr(&s);

    println!("{:#?}", freq);
}

fn frq_ctr(s: &str) -> HashMap<&str, i32> {
    let mut wc = HashMap::new();

    for word in s.split_whitespace() {
        wc.entry(word).and_modify(|c| *c += 1).or_insert(1);
    }

    // to return top 3
    // The easiest thing I can think of is to store values in a vector and
    // get all teh corresponding keys for those
    let mut vec: Vec<i32> = Vec::new();
    for val in wc.values() {
        vec.push(*val);
    }

    //sort in descending order
    vec.sort_by(|a, b| b.cmp(a));

    let mut twc = HashMap::new();
    //i'll only print one word (if they matchtoo)

    //can add checks and make it tighter but.. it is what it is
    let mut ctr = 0;
    while ctr < 3 {
        //get doesnt cause the program to panic
        let v = vec.get(ctr);
        let temp: i32;
        match v {
            None => break,
            Some(i) => temp = *i,
        };

        let k = wc
            .iter()
            .find_map(|(key, val)| if *val == temp { Some(*key) } else { None });

        match k {
            None => None,
            Some(i) => twc.insert(i, vec[ctr]),
        };
        ctr += 1;
    }
    twc
}

//look at this gpt code and be ashamed of yourself ... fucking retardd

// use std::collections::HashMap;
//
// fn main() {
//     let s = "hi my name is chika chika slim shady slim slim";
//     let top = top_3_words(s);
//
//     println!("{:#?}", top);
// }
//
// fn top_3_words(s: &str) -> Vec<(&str, i32)> {
//     let mut wc = HashMap::new();
//
//     // Count frequencies
//     for word in s.split_whitespace() {
//         *wc.entry(word).or_insert(0) += 1;
//     }
//
//     // Convert to vector of (word, count)
//     let mut pairs: Vec<(&str, i32)> = wc.into_iter().collect();
//
//     // Sort by frequency (descending)
//     pairs.sort_by(|a, b| b.1.cmp(&a.1));
//
//     // Take top 3
//     pairs.into_iter().take(3).collect()
// }
