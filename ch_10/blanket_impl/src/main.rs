/*
- Blanket impl + extension trait
- define a trait Describe { fn describe(&self) -> String;}
- write a blanket impl: impl<T: Display + Debug>
- Describe for T -- so every type that is both Display and Debug gets a free describe()
  that returns "display: {self}, debug:{self:?}"
- verify it works on i32, String, Vec<i32>, Vec2 (from operator overloading).

- Then define a second trait "IterDescribe" for anything that is IntIterator where
  Item: Describe -- giving it a fn describe_all(&self) -> Vec<String>, impl as blanket
*/

use std::fmt::{Debug, Display};
use std::iter::IntoIterator;

trait Describe {
    fn describe(&self) -> String;
}

impl<T: Display + Debug> Describe for T {
    fn describe(&self) -> String {
        //should this not print and then return the string ?
        format!("display: {self}, debug:{self:?}")
    }
}

trait IterDescribe {
    fn describe_all(&self) -> Vec<String>;
}

impl<T: IntoIterator> IterDescribe for T {
    fn describe_all(&self) -> Vec<String>
    //each element of the iterator implements describe
    //not the whole iterator itself
    where
        T::Item: Describe,
    {
        let mut res_iter: Vec<String> = Vec::new();

        //types implementing IntoIterator will work with
        //the for loop syntax
        for res in self {
            res_iter.push(res.describe());
        }

        res_iter
    }
}

fn main() {
    //verify working of display on i32
    let num: i32 = 1;
    let n = num.describe();
    println!("{n}");

    let s = String::from("himynameischikachikaslimshady");
    println!("{}", s.describe());

    //doesnt implement display trait, so doesn't work
    // let v: Vec<i32> = vec![1,2,4];
    // println!("{}", v.describe());

    //there's no debug implemented to vec2. If we implement it, it'll work.
    //too much work to do, so i'm skipping that part

    let s_iter = String::from("himynameischikachikaslimshady");
    //Converts a String into an iterator over the chars of the string.
    let s_iter = s_iter.into_chars();

    let s_vec = s_iter.describe_all();
    println!("{:?}", s_vec);
}
