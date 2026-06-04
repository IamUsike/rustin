// - `push()`
// - `pop() -> Option<T>`
// - `peek() -> Option<&T>`
// - `is_empty() -> bool`
// - `size() -> usize`
// Test with `Stack<i32>` and `Stack<String>`.

use std::fmt::Display;

struct Stack<T: Display> {
    data: Vec<T>,
}

fn main() {
    let mut stack = Stack { data: vec![1] };
    stack.push(2);
    stack.peek();

    println!("{}", stack.is_empty());
    println!("{}", stack.size());
}

impl<T: Display> Stack<T> {
    fn push(&mut self, val: T) -> () {
        self.data.push(val)
    }

    fn peek(&self) {
        if let Some(last_element) = self.data.last() {
            println!("{last_element}")
        } else {
            println!("Empty Stack")
        }
    }

    fn pop(&mut self) -> Option<T> {
        //lollll
        self.data.pop()
    }

    fn is_empty(&self) -> bool {
        if self.data.len() == 0 { true } else { false }
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}
