fn main() {
    let s = "hello        world";
    let rev = reverse_words(&s);
    println!("rev word = {rev}");
}

fn reverse_words(s: &str) -> String {
    let mut s: Vec<&str> = s.split_whitespace().collect();
    s.reverse();

    let str: String = s.join(" ");

    str
}
