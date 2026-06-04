fn main() {
    //Case 1: Both match prefix, return longer one
    // let s1 = "apple";
    // let s2 = "application";
    // let prefix = "app";
    //
    // assert_eq!(longest_starting_with(s1, s2, prefix), "application");

    //     //Case 2: Only s1 matches
    //     let s1 = "apple";
    //     let s2 = "banana";
    //     let prefix = "app";
    //
    //     assert_eq!(longest_starting_with(s1, s2, prefix), "apple");

    let s1 = "banana";
    let s2 = "application";
    let prefix = "app";

    assert_eq!(longest_starting_with(s1, s2, prefix), "application");
}

//theres 3 cases
//none of them contain prefix
//one of them contains prefix
//both of them contain prefix
fn longest_starting_with<'a>(s1: &'a str, s2: &'a str, prefix: &str) -> &'a str {
    if s1.starts_with(prefix) && s2.starts_with(prefix) {
        if s1.len() >= s2.len() { s1 } else { s2 }
    } else if s1.starts_with(prefix) {
        s1
    } else if s2.starts_with(prefix) {
        s2
    } else {
        s1
    }
}
