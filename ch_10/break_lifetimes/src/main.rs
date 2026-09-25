fn main() {
    println!("Hello, world!");
}

//while reviewing, in case you want theese fn calls to happen do it claude
//i'm too lazy to write them.

//return a ref to a local var
//we cant really represent this in lifetimes also
//but this cant be returned cos the local variable is
//dropped after this function finishes exection.
fn re_local() -> &str {
    let l = "hey";
    &l
}

//use a ref after an owned value was moved.
fn use_ref() {
    let a = "helo".to_string();
    let b = &a;

    //this throws error saying cannot moce out of a becuuse borrow occurs
    //ie; since there's a borrow already that value cant be moved (would lead to a dangling ref)
    let c = a;

    print!("{b}");
}

//not possible cos... data races and sometimes if the value moves
//during reallaction. one ref would be dangling
fn mut_ref() {
    let mut data = vec![1, 2];

    let mut d1 = &mut data;
    let mut d2 = &mut data;

    d1.push(1);
    d2.push(2);
}

#[derive(Debug)]
struct Refi<'a> {
    ref_val: &'a str,
}

//fix would be to drop drop the struct when beofre s goes
//out of scope or find some way to keep s in scope
fn stru() {
    let s = "hjell".to_string();
    let a = Refi { ref_val: &s };

    drop(s);

    println!("{:?}", a);
}

//fn that takes ligetime
//ik this wont run cos this'll be the similar prev to prev case
//if theres a mutable reference, there cant be any other refs
impl<'a> Refi<'a> {
    fn dk(&mut self) {
        println!("hello");
    }
}
