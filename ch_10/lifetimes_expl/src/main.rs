fn main() {
    println!("{}", first_word("hello world"));
    println!("{}", longer("short", "longer one"));
    println!("{:?}", first(&[1, 2, 3]));
    let split = StrSplit { remainder: "a,b,c" };
    println!("{}", split.remainder);
    println!("{}", substr("hello world", 6, 5));
}

//too lazy to write call these fns.
//claude, if you reviewing these just add the
//calls in the main function and nothing more

//no need to explicitly define here because of lifetime elision but.
//this means that the output will live for as long the the input.
fn first_word<'a>(s: &'a str) -> &'a str {
    let (first, _) = s.split_once(" ").unwrap();
    first
}

//the output will live as long as both s1 and s2 exist
fn longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        return s1;
    }

    s2
}

//we declare a generic lifetim 'a  and a generic param T
//The output (of type T) lives for as long as the input
fn first<'a, T>(silce: &'a [T]) -> Option<&'a T> {
    Some(&silce[0])
}

//the instance of this struct will be kept alive until
//the value referred to in the remainder field exists
struct StrSplit<'a> {
    remainder: &'a str,
}

//the output will be alive as long as the input param (s) exists
fn substr<'a>(s: &'a str, start: usize, len: usize) -> &'a str {
    &s[start..start + len]
}
