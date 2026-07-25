enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

use List::{Cons, Nil};

impl<T> List<T> {
    fn new() -> Self {
        Nil
    }

    fn push_front(self, value: T) -> Self {
        Cons(value, Box::new(self))
    }

    fn len(&self) -> usize {
        match self {
            Cons(_, rest) => 1 + rest.len(),
            Nil => 0,
        }
    }

    fn contains(&self, target: &T) -> bool
    where
        T: PartialEq,
    {
        match self {
            Cons(value, rest) => value == target || rest.contains(target),
            Nil => false,
        }
    }

    fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        match self {
            Cons(value, rest) => {
                let mut v = vec![value.clone()];
                v.extend(rest.to_vec());
                v
            }
            Nil => Vec::new(),
        }
    }
}

fn main() {
    let list = List::new().push_front(3).push_front(2).push_front(1);

    println!("len: {}", list.len()); // 3
    println!("contains(2): {}", list.contains(&2)); // true
    println!("contains(5): {}", list.contains(&5)); // false
    println!("to_vec: {:?}", list.to_vec()); // [1, 2, 3]
}

/*
 * - Why Box<List<T>> is required: List<T> is recursive (it contains itself), so without indirection
 * the compiler can't compute a finite size for it. Box puts the recursive part on the heap, giving
 * List<T> a fixed size (a tag + a pointer).
 *
 * - push_front takes self by value, not &mut self. Since the list is immutable-style (each node owns the next), prepending means consuming the old list and wrapping it in a new Cons — you can't mutate in place without extra ceremony (Option::take, etc.).
 *
 *
 *  - len, contains, to_vec all recurse structurally on match self. Since they take &self, matching gives you &T and &Box<List<T>> — you never need to .clone() the whole list, only individual elements when building to_vec.
 *
 *
 *  - to_vec's recursion is naive (a new Vec allocated at every level, then extended). Fine for correctness; if you want tail-recursive/accumulator style for efficiency, add a helper that takes a &mut Vec<T> and pushes into it instead of returning new vecs each time — worth trying as a follow-up.
 *
 *
 *  - One recursion gotcha: rest.len() works because rest: &Box<List<T>> auto-derefs to &List<T> via Deref, so you can call List methods on it directly without manual *rest or .as_ref().
 */
