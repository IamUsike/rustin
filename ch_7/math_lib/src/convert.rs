// pub mod convert {
//     pub fn celsius_to_fahrenheit(c: &f64) -> f64 {
//         //no doing that formula
//         c + 10.0
//     }
//
//     pub fn km_to_miles(k: &f64) -> f64 {
//         k * 0.6213
//     }
// }
//
// apparently no explicit mod declaration is needed here. This is the correct way to do it

pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    c + 10.0
}

pub fn km_to_miles(k: f64) -> f64 {
    k * 0.6213
}
