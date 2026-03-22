fn main() {
    let s = String::from("sup beach");
    //this cant be done because each value in rust has only one owner. and since string
    //stays on the heap, the value is moved to rhe function params
    //and 's' will no longer own the string
    // take_ownership(s);
    // println!("{s}");

    //this will not throw an error cos clone creates a deep copy and the ownership of
    // the string remains with 's' itself
    // take_ownership(s.clone());
    // println!("{s}");

    //here, ownership is still preserved as we are only passin the pointer to 's'
    take_reference(&s);
    println!("{s}");
}

// a) Take ownership
fn take_ownership(s: String) {
    println!("ownership fn prints {s}");
}

fn take_reference(s: &str) {
    println!("ref fn prints {s}");
}
