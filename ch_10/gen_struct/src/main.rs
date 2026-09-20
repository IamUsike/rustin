/* Generic Struct with multiple type params
- Define a Pair<K, V> Struct.
- Add: fn new(key: K, val: V) -> Self, fn key(&self) -> &K, fn val(&self) -> &V,
fn swap(&self) -> Pair<V, K>.
- the add a method fn display(&self) only when both K and V implement display.
- use an impl block with where clause. Test with Pair<String, i32> and Pair<&str, bool>
*/
// use std::cmp::PartialOrd;
use std::fmt::{Debug, Display};
#[derive(Debug)]
struct Pair<K, V> {
    key: K,
    val: V,
}

impl<K: Clone, V: Clone> Pair<K, V> {
    fn new(key: K, val: V) -> Self {
        Pair { key, val }
    }

    fn key(&self) -> &K {
        &self.key
    }

    fn val(&self) -> &V {
        &self.val
    }

    //assuming we need to create a new pair. Cos tried swapping in the original,
    //but since the type is already K, V idk how to make the type to V, K.
    fn swap(&self) -> Pair<V, K>
    where
        V: Debug,
        K: Debug,
    {
        Pair {
            key: self.val.clone(),
            val: self.key.clone(),
        }
    }

    fn display(&self)
    where
        K: Display,
        V: Display,
    {
        println!("displaynn");
        println!("{} : {}", self.key, self.val);
    }
}

fn main() {
    let pair = Pair::new('a', 0);
    let val = pair.val();
    println!("val is {val}");
    let key = pair.key();
    println!("key is {key}");
    println!("swappopp");
    //can shadow(override pair here), to get the inplace replace effect
    let p = pair.swap();
    println!("{:?}", p);

    let val = pair.val();
    println!("val is {val}");
    let key = pair.key();
    println!("key is {key}");
    pair.display();
    p.display();
}
