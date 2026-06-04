fn longest_starting_with<'a>(s1: &'a str, s2: &'a str, prefix: &str) -> &'a str {
    let s1_match = s1.starts_with(prefix);
    let s2_match = s2.starts_with(prefix);

    match (s1_match, s2_match) {
        (true, true) => {
            if s1.len() >= s2.len() {
                s1
            } else {
                s2
            }
        }
        (true, false) => s1,
        (false, true) => s2,
        (false, false) => s1,
    }
}
