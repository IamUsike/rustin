fn main() {
    println!("Hello, world!");
    //this doesnt panic as ieee 754 standard is followed | x will be inf
    // let x = 5.0 / 0.0;
    let sq = compute(25.0, 1.0);
    println!("{:?}", sq);
}

fn safe_divide(a: f64, b: f64) -> Option<f64> {
    match b {
        0.0 => None,
        _ => Some(a / b),
    }
}

fn safe_sqrt(x: f64) -> Option<f64> {
    if x < 0.0 { None } else { Some(x.sqrt()) }
}

fn compute(a: f64, b: f64) -> Option<f64> {
    let div = safe_divide(a, b);
    match div {
        None => None,
        Some(i) => safe_sqrt(i),
    }
}

//improv

// fn compute(a: f64, b: f64) -> Option<f64> {
//     if let Some(div) = safe_divide(a, b) {
//         safe_sqrt(div)
//     } else {
//         None
//     }
// }
