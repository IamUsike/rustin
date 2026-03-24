fn main() {
    let mut v = vec![1, 2, 3];

    double_all(&mut v);
    let sum = sum_slice(&v);

    println!("vector is {:?}", v);
    println!("sum = {sum}");
}

fn double_all(v: &mut Vec<i32>) -> () {
    for i in v {
        *i *= 2;
    }
}

fn sum_slice(s: &[i32]) -> i32 {
    let mut sum = 0;
    for i in s {
        sum += i;
    }

    sum
}
