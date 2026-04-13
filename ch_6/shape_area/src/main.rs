enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle(f64, f64, f64),
}

fn main() {
    let circle = Shape::Circle(6.0);
    let ar = area(&circle);
    println!("{ar}");

    let rect = Shape::Rectangle(4.0, 5.0);
    println!("rect: {}", area(&rect));
}

fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle(r) => 3.14 * r * r,
        Shape::Rectangle(l, b) => l * b,
        Shape::Triangle(a, b, c) => {
            let s = (a + b + c) / 2.0;
            let h = s * (s - a) * (s - b) * (s - c);
            h.sqrt()
        }
    }
}

// Pattern matching in Rust is smarter than operators — it auto-derefs before binding variables.

//else

// fn area(shape: &Shape) -> f64 {
//     match shape {
//         Shape::Circle(&r) => 3.14 * r * r,
//         Shape::Rectangle(&l, &b) => l * b,
//         Shape::Triangle(&a, &b, &c) => {
//             let s = (a + b + c) / 2.0;
//             let h = s * (s - a) * (s - b) * (s - c);
//             h.sqrt()
//         }
//     }
// }
