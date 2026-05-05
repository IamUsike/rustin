use std::{collections::HashMap, vec};

fn main() {
    let words: Vec<&str> = vec!["eat", "tea", "tan", "ate", "nat", "bat"];
    an_gr(words);
}

fn an_gr(words: Vec<&str>) -> Vec<Vec<&str>> {
    //create a hashmap.
    //- loop through each word in the vec
    //- sort the word alphabetically(not in place)
    //- the sorted word is the `key` of the hashmap
    //- the word(from vec) will be the value.
    //- if the key exists append the word to the vector
    //- else create a new entry with the respective kvp

    // let mut anagrams = HashMap::new();

    let mut ang: HashMap<String, Vec<&str>> = HashMap::new();

    for word in words {
        let mut key: Vec<char> = word.chars().collect();
        key.sort();
        let key: String = key.into_iter().collect();

        //if the entry is occupied push the word into it else initialize a vector with the word
        ang.entry(key)
            .and_modify(|key| key.push(word))
            .or_insert_with(|| vec![word]);
    }

    println!("{:?}", ang);

    //this borrows ang
    let anagrams: Vec<Vec<&str>> = ang.into_values().collect();

    println!("{:?}", anagrams);
    anagrams
}
