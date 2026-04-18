//look at convert.rs and come back |
pub mod stats {
    pub fn mean(vec: &Vec<f64>) -> f64 {
        let total: f64 = vec.iter().sum(); //creates an iterable over immutable refference 
        total / (vec.len() as f64)
    }

    pub fn median(vec: &Vec<f64>) -> f64 {
        let no_of_terms = vec.len() / 2;
        if no_of_terms == 0 {
            let med = vec[no_of_terms / 2] + vec[no_of_terms / 2 + 1];
            med / 2.0
        } else {
            vec[(no_of_terms + 1) / 2]
        }
    }

    // pub fn mode(vec: &Vec<f64>) -> f64 {
    // can be done with hashset but i;m lazy
    // }
}

//vectors are not iterators. They are collections
//let v = vec![1, 2, 3];
// for x in v {
//     println!("{x}");
// }
// rust calls .into_iter() under the hood
