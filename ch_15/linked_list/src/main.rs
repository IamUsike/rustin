use std::fmt::Debug;

#[derive(Debug)]
enum List<T> {
    Cons(T, Box<Self>),
    Nil,
}

fn main() {
    let l1 = List::Cons(5, Box::new(List::Cons(6, Box::new(List::Nil))));
    let l1 = l1.push_front(5);

    // let l1: List<i32> = List::Nil;

    println!("{}", l1.len());
    println!("{}", l1.contains(&0));
    println!("{:?}", l1.to_vec());
}

impl<T> List<T> {
    //need to add a value to the beginning of the cons list
    //Since 2nd element is of the type List.
    //
    //Things to try
    //1. implement clone method on list.
    //2. make the param within box &Self (cant ques)
    //3. use lifetime and allow val to live for as long as self lives | this wont work cos we aren't
    //   changing the lifetimes of any params but specifying the relation. We cant guarantee that
    //   val will live until the end of l1 plus the enum definition doesn't define any references

    fn push_front(self, val: T) -> Self {
        List::Cons(val, Box::new(self))
    }

    fn len(&self) -> usize
    where
        T: Debug,
    {
        let mut count = 0;
        let mut curr = self;

        while let List::Cons(_, next) = curr {
            count += 1;
            curr = next;
        }

        count
    }

    fn contains(&self, val: &T) -> bool
    where
        T: Ord,
    {
        let mut curr = self;
        while let List::Cons(ele, next) = curr {
            if *val == *ele {
                return true;
            }
            curr = next;
        }

        false
    }

    fn to_vec(&self) -> Vec<T>
    where
        T: Copy,
    {
        let mut vec_list = Vec::new();
        let mut curr = self;

        while let List::Cons(ele, next) = curr {
            vec_list.push(ele.clone());
            curr = next;
        }

        vec_list
    }
}
