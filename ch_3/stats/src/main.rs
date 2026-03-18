fn main() {
    let arr: [i32; 8] = [1, 2, 3, 4, 5, 6, -7, 8];
    let min: i32 = min(arr);
    println!("{min}");
    let max: i32 = max(arr);
    println!("{max}");
    let mean: i32 = mean(arr);
    println!("{mean}");

    println!("range: {min} to {max}");
}

//https://users.rust-lang.org/t/why-i-cant-pass-an-array-as-an-argument-to-another-function/99226
fn min(arr: [i32; 8]) -> i32 {
    let mut min = i32::MAX;
    for a in arr {
        if a < min {
            min = a
        }
    }

    min
}

fn max(arr: [i32; 8]) -> i32 {
    let mut max = i32::MIN;
    for a in arr {
        if a > max {
            max = a
        }
    }

    max
}

fn mean(arr: [i32; 8]) -> i32 {
    let mut sum: i32 = 0;
    for a in arr {
        if sum >= i32::MAX {
            break;
        }
        sum += a;
    }

    return sum / 8;
}

//ownership and references gottilla
