fn main() {
    //     let str = vec!["42", "foo", "17", "bar", "100"];
    //     let it = str.iter().filter_map(|s| s.parse::<i32>().ok());
    //
    //     dbg!(it);

    let a = ["1", "two", "NaN", "four", "5"];

    let iter: Vec<i32> = a
        .iter()
        .filter_map(|s| s.parse::<i32>().ok().map(|x| x * 2))
        .collect();

    dbg!(iter);
    // assert;_eq!(iter.next(), Some(1));
    // assert_eq!(iter.next(), Some(5));
    // assert_eq!(iter.next(), None);
}
