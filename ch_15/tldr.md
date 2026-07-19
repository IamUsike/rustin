# 15 Smart Pointers

15.1 Using Box<T> to point to Data on the Heap
15.2 Treating Smart Pointers Like Regular References
15.3 Running Code on Cleanup with the Drop Trait
15.4 Rc<T>, the Reference Counted Smart Pointer
15.5 RefCell<T> and the iterator Mutability Pattern
15.6 Reference Cycles can Leak Memory

looks like this'll be a long chapter

- A pointer is a general concept for a variable that contains an address in memory.
- The most common type is a reference. They don’t have any special capabilities other than referring to data, and they have no overhead.

- _Smart Pointers_ are data structures that act like a pointer but also have additional metadata and capabilities.
- In Rust, with its concept of ownership and borrowing, there is an additional difference between references and smart pointers: While references only borrow data, in many cases smart pointers own the data they point to.

- Smart pointers are usually implemented using structs. Unlike an ordinary struct, smart pointers implement the Deref and Drop traits. The Deref trait allows an instance of the smart pointer struct to behave like a reference so that you can write your code to work with either references or smart pointers. The Drop trait allows you to customize the code that’s run when an instance of the smart pointer goes out of scope.

---

## Using `Box<T>` to Point to Data on the Heap

Box is the most straight forward pointer. Type is written as `Box<T>`. _Boxes_ allow us to store data on the heap rather than the stack. What remains on the stack is the pointer to the heap.

Boxes dont have performance overhead other than storing data on the heap instead of the stack. But they dont have many extra capabilities either. They are often used in:

- When you have a type whose size can’t be known at compile time, and you want to use a value of that type in a context that requires an exact size
- When you have a large amount of data, and you want to transfer ownership but ensure that the data won’t be copied when you do so
- When you want to own a value, and you care only that it’s a type that implements a particular trait rather than being of a specific type

1 in a later sec of this ch and 3 in ch-18.
-> To transfer ownership of a large amount of data takes a long time cos it is being moved in the stack. Instead, we can store all this data on the heap using box and just move the ownership of the pointer.

### Storing Data on the Heap

syntax and interacting with values stored within a box.

-> Store an i32 value on the heap

```rust
fn main() {
    //points to the value 5 stored on the heap
    let b = Box::new(5);
    // 5 will be printed (dispaly)
    println!("b = {b}");
}
```

- just like an owned value, when a box goes out of scope(at the end of main in this case) as does b. Both heap and stack will be deallocated

### Enabling Recursive Types with Boxes

A value of a recursive type can have another value of the same type as part of itself. Recursive types pose an issue because Rust needs to know at compile time how much space a type takes up. However, the nesting of values of recursive types could theoretically continue infinitely, so Rust can’t know how much space the value needs. Because boxes have a known size, we can enable recursive types by inserting a box in the recursive type definition.

As an example of a recursive type, let’s explore the cons list. This is a data type commonly found in functional programming languages. The cons list type we’ll define is straightforward except for the recursion; therefore, the concepts in the example we’ll work with will be useful anytime you get into more complex situations involving recursive types.

#### Understanding the Cons List

Name comes from construct function (lisp language) -> Constructs a new pair from its args. By calling cons on a pair consisting of a value and another pair, we can construct cons lists made up of recursive pairs.

pseudocode rep of a cons list containing the list 1,2,3 with each pair in parenthesis

```
(1, (2, (3, Nil)))
```

Each item in a cons list contains two elements: the value of the current item and of the next item. The last item in the list contains only a value called Nil without a next item. A cons list is produced by recursively calling the cons function.

(skipping many things: read book)

Because a Box<T> is a pointer, Rust always knows how much space a Box<T> needs: A pointer’s size doesn’t change based on the amount of data it’s pointing to. This means we can put a Box<T> inside the Cons variant instead of another List value directly. The Box<T> will point to the next List value that will be on the heap rather than inside the Cons variant. Conceptually, we still have a list, created with lists holding other lists, but this implementation is now more like placing the items next to one another rather than inside one another.

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use crate::List::{Cons, Nil};

fn main() {
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
}
```

The Cons variant needs the size of an i32 plus the space to store the box’s pointer data. The Nil variant stores no values, so it needs less space on the stack than the Cons variant. We now know that any List value will take up the size of an i32 plus the size of a box’s pointer data. By using a box, we’ve broken the infinite, recursive chain, so the compiler can figure out the size it needs to store a List value. Figure 15-2 shows what the Cons variant looks like now.

---

## Treating Smart Pointers Like Regular References

implementing `Deref` trait allows us to customize the behavior of _dereference operator_ `*`.
`Deref` can be implemented in such a way that a smart pointer can be treated like a regular reference, we can write code on that operates on references and use that code with smart pointers too.

### Following the Reference to the value

- A regular reference is a type of pointer, and one way to think of a pointer is as an arrow to a value stored somewhere else.

```rust
fn main() {
    let x = 5;
    let y = &x;

    assert_eq!(5, x);
    assert_eq!(5, *y);
}
```

### Using `Box<T>` Like a Reference

We can write an example similar to the previous using box

```rust
fn main() {
    let x = 5;
    let y = Box::new(x);

    assert_eq!(5, x);
    assert_eq!(5, *y);
}
```

The main difference between Listing 15-7 and Listing 15-6 is that here we set y to be an instance of a box pointing to a copied value of x rather than a reference pointing to the value of x. In the last assertion, we can use the dereference operator to follow the box’s pointer in the same way that we did when y was a reference. Next, we’ll explore what is special about Box<T> that enables us to use the dereference operator by defining our own box type.

### Defining Our Own Smart Pointer

Let's build a wrapper similar to `Box<T>` type provided by the standard library to experience how smart pointer types behave differently from references by default.

> Note: There’s one big difference between the MyBox<T> type we’re about to build and the real Box<T>: Our version will not store its data on the heap. We are focusing this example on Deref, so where the data is actually stored is less important than the pointer-like behavior.

The Box<T> type is ultimately defined as a tuple struct with one element.

```rust
struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}
```

if we write a main fn for this and try to compile it:

```rust
fn main() {
    let x = 5;
    let y = MyBox::new(x);

    assert_eq!(5, x);
    assert_eq!(5, *y);
}
```

```
$ cargo run
   Compiling deref-example v0.1.0 (file:///projects/deref-example)
error[E0614]: type `MyBox<{integer}>` cannot be dereferenced
  --> src/main.rs:14:19
   |
14 |     assert_eq!(5, *y);
   |                   ^^ can't be dereferenced

For more information about this error, try `rustc --explain E0614`.
error: could not compile `deref-example` (bin "deref-example") due to 1 previous error
```

Our MyBox<T> type can’t be dereferenced because we haven’t implemented that ability on our type. To enable dereferencing with the `*` operator, we implement the Deref trait.

#### Implementing the `Deref` Trait

To implement a trait, we need to provide implementations for the trait's required methods. The Deref trait, provided by the standard library, requires us to implement one method named deref that borrows self and returns a reference to the inner data.

```rust
use std::ops::Deref;

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
```

The type Target = T; syntax defines an associated type for the Deref trait to use. Associated types are a slightly different way of declaring a generic parameter, but you don’t need to worry about them for now; we’ll cover them in more detail in Chapter 20.

#### Using Deref Coercion in Functions and Methods

_Deref Coercion_ converts a reference to a type that implements the `Deref` trait into a reference to another type.

Deref coercion converts a reference to a type that implements the Deref trait into a reference to another type. For example, deref coercion can convert &String to &str because String implements the Deref trait such that it returns &str. Deref coercion is a convenience Rust performs on arguments to functions and methods, and it works only on types that implement the Deref trait. It happens automatically when we pass a reference to a particular type’s value as an argument to a function or method that doesn’t match the parameter type in the function or method definition. A sequence of calls to the deref method converts the type we provided into the type the parameter needs.

Deref coercion was added to Rust so that programmers writing function and method calls don’t need to add as many explicit references and dereferences with & and \*. The deref coercion feature also lets us write more code that can work for either references or smart pointers.

### Handling Deref Coercion with Mutable References

Similar to how you use the `Deref` trait to override the `*` operator on immutable references, you can use the `DerefMut` trait to override the `*` operator on mutable references.

Rust does deref coercion when it finds types and trait implementations in three cases:

1. From `&T` to `&U` when `T: Deref<Target = U>`
2. From `&mut T` to `&mut U` when `T: DerefMut<Target = U>`
3. From `&mut T` to `&U` when `T: Deref<Target = U>`

The first two cases are the same except that the second implements mutability. The first case states that if you have a `&T`, and `T` implements `Deref` to some type `U`, you can get a `&U` transparently. The second case states that the same deref coercion happens for mutable references.

The third case is trickier: Rust will also coerce a mutable reference to an immutable one. But the reverse is _not_ possible: immutable references will never coerce to mutable references. Because of the borrowing rules, if you have a mutable reference, that mutable reference must be the only reference to that data (otherwise, the program wouldn’t compile). Converting one mutable reference to one immutable reference will never break the borrowing rules. Converting an immutable reference to a mutable reference would require that the initial immutable reference is the only immutable reference to that data, but the borrowing rules don’t guarantee that. Therefore, Rust can’t make the assumption that converting an immutable reference to a mutable reference is possible.

---

## Running Code on Cleanup with the `Drop` Trait

`Drop` trait lets us customize what happens when a value goes out of scope. We can provide an implementation for the `Drop` trait on any type.

We intro in context of smart pointers here cos it's almost always used in that context.

Eg: when a box is dropped, it will deallocate the space on the heap that the box points to.

In some languages, for some types, the programmer must call code to free memory or resources every time they finish using an instance of those types. Examples include file handles, sockets, and locks. If the programmer forgets, the system might become overloaded and crash. In Rust, you can specify that a particular bit of code be run whenever a value goes out of scope, and the compiler will insert this code automatically. As a result, you don’t need to be careful about placing cleanup code everywhere in a program that an instance of a particular type is finished with—you still won’t leak resources!

You specify the code to run when a value goes out of scope by implementing the Drop trait. The Drop trait requires you to implement one method named drop that takes a mutable reference to self. To see when Rust calls drop, let’s implement drop with println! statements for now.

Let's implement a `CustomSmartPointer` struct whose only custom functionality is that it prints shi when instance goes out of scope.

```rust
struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data);
    }
}

fn main() {
    let c = CustomSmartPointer {
        data: String::from("my stuff"),
    };
    let d = CustomSmartPointer {
        data: String::from("other stuff"),
    };
    println!("CustomSmartPointers created");
}
```

- d is dropped first and then c (rev order)
- rust automatically called `drop` for us when our instances went out of scope.
- we cant call `Drop` trait's `drop` (`c.drop()`) method directly. Instead, we've to call `std::mem::drop` fn provided by the standard library to force a value to droppped before it goes out of scope.

The std::mem::drop function is different from the drop method in the Drop trait. We call it by passing as an argument the value we want to force-drop. The function is in the prelude, so we can modify main in Listing 15-15 to call the drop function, as shown in Listing 15-16.

```rust
fn main() {
    let c = CustomSmartPointer {
        data: String::from("some data"),
    };
    println!("CustomSmartPointer created");
    drop(c);
    println!("CustomSmartPointer dropped before the end of main");
}
```

---

## `Rc<T>, the Reference Counted Smart Pointer`

In majority of cases, ownership is clear: You know exactly which variable owns a given value. However, there are cases when a single value might have multiple owners, for example: in graph data structures multiple edges might point to the same node, and that node is conceptually owned by all of the edges that point to it. A node shouldn’t be cleaned up unless it doesn’t have any edges pointing to it and so has no owners.

You have to enable multiple ownership explicitly by using the Rust type Rc<T>, which is an abbreviation for reference counting. The Rc<T> type keeps track of the number of references to a value to determine whether or not the value is still in use. If there are zero references to a value, the value can be cleaned up without any references becoming invalid.

We use the Rc<T> type when we want to allocate some data on the heap for multiple parts of our program to read and we can’t determine at compile time which part will finish using the data last. If we knew which part would finish last, we could just make that part the data’s owner, and the normal ownership rules enforced at compile time would take effect.

Note that Rc<T> is only for use in single-threaded scenarios. When we discuss concurrency in Chapter 16, we’ll cover how to do reference counting in multithreaded programs.

### Sharing Data

```rust
enum List {
    Cons(i32, Rc<List>),
    Nil,
}

use crate::List::{Cons, Nil};
use std::rc::Rc;

fn main() {
    let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
    let b = Cons(3, Rc::clone(&a));
    let c = Cons(4, Rc::clone(&a));
}
```

### Cloning to increase the Reference Count

```rust
// --snip--

fn main() {
    let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
    println!("count after creating a = {}", Rc::strong_count(&a));
    let b = Cons(3, Rc::clone(&a));
    println!("count after creating b = {}", Rc::strong_count(&a));
    {
        let c = Cons(4, Rc::clone(&a));
        println!("count after creating c = {}", Rc::strong_count(&a));
    }
    println!("count after c goes out of scope = {}", Rc::strong_count(&a));
}
```

```
$ cargo run
   Compiling cons-list v0.1.0 (file:///projects/cons-list)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.45s
     Running `target/debug/cons-list`
count after creating a = 1
count after creating b = 2
count after creating c = 3
count after c goes out of scope = 2
```

Via immutable references, Rc<T> allows you to share data between multiple parts of your program for reading only. If Rc<T> allowed you to have multiple mutable references too, you might violate one of the borrowing rules discussed in Chapter 4: Multiple mutable borrows to the same place can cause data races and inconsistencies. But being able to mutate data is very useful! In the next section, we’ll discuss the interior mutability pattern and the RefCell<T> type that you can use in conjunction with an Rc<T> to work with this immutability restriction.

---

## `Refcell<T>` and the Interior Mutability Pattern

- _Interior mutability_ is a design pattern that allows you to mutate data even when there are immutable references to that data.
- To mutate data, the pattern uses `unsafe` code inside a data structure to bend Rust's usual rules that govern mutation and borrowing.
- Unsafe code indicates to the compiler that we're checking unsafe code manually instead of the compiler doing it for us.

> We can use types that use the interior mutability pattern only when we can ensure that the borrowing rules will be followed at runtime, even though the compiler can’t guarantee that. The unsafe code involved is then wrapped in a safe API, and the outer type is still immutable.

### Enforcing Borrowing Rules at Runtime

Unlike Rc<T>, the RefCell<T> type represents single ownership over the data it holds. So, what makes RefCell<T> different from a type like Box<T>? Recall the borrowing rules you learned in Chapter 4:

- At any given time, you can have either one mutable reference or any number of immutable references (but not both).
- References must always be valid.

With references and Box<T>, the borrowing rules’ invariants are enforced at compile time. With RefCell<T>, these invariants are enforced at runtime. With references, if you break these rules, you’ll get a compiler error. With RefCell<T>, if you break these rules, your program will panic and exit.

Similar to Rc<T>, RefCell<T> is only for use in single-threaded scenarios and will give you a compile-time error if you try using it in a multithreaded context. We'll talk about how to get the functionality of RefCell<T> in a multithreaded program in Chapter 16.

Here is a recap of the reasons to choose Box<T>, Rc<T>, or RefCell<T>:
• Rc<T> enables multiple owners of the same data; Box<T> and RefCell<T> have single owners.
• Box<T> allows immutable or mutable borrows checked at compile time; Rc<T> allows only immutable borrows checked at compile time; RefCell<T> allows immutable or mutable borrows checked at runtime.
• Because RefCell<T> allows mutable borrows checked at runtime, you can mutate the value inside the RefCell<T> even when the RefCell<T> is immutable.

Mutating the value inside an immutable value is the interior mutability pattern.

---

### Using Interior Mutability

- The borrowing rules dont let us borrow an immutable value mutably
- There are situations in which it would be useful for a value to mutate itself in its methods but appear immutable to other parts of code.

- Using `RefCell<T>` is one way to get the ability to have interior mutability, but `RefCell<T>` doesn’t get around the borrowing rules completely: The borrow checker in the compiler allows this interior mutability, and the borrowing rules are checked at runtime instead. If you violate the rules, you’ll get a panic! instead of a compiler error.

#### Testing with Mock Objects

- Terms throw: `test double`, `mock Objects(record whats happening during tests)`

Rust doesn’t have objects in the same sense as other languages have objects, and Rust doesn’t have mock object functionality built into the standard library as some other languages do. However, you can definitely create a struct that will serve the same purposes as a mock object.

too many things happening :crayyy

#### Tracking Borrows at Runtime

When creating immutable and mutable references, we use the `&` and `&mut` syntax, respectively. With `RefCell<T>`, we use the `borrow` and `borrow_mut` methods, which are part of the safe API that belongs to `RefCell<T>`. The `borrow` method returns the smart pointer type `Ref<T>`, and `borrow_mut` returns the smart pointer type `RefMut<T>`. Both types implement `Deref`, so we can treat them like regular references.

The `RefCell<T>` keeps track of how many `Ref<T>` and `RefMut<T>` smart pointers are currently active. Every time we call `borrow`, the `RefCell<T>` increases its count of how many immutable borrows are active. When a `Ref<T>` value goes out of scope, the count of immutable borrows goes down by 1. Just like the compile-time borrowing rules, `RefCell<T>` lets us have many immutable borrows or one mutable borrow at any point in time.

if we violate these rules the program will panic.

### Allowing multiple Owners of Mutable Data

A common way to use RefCell<T> is in combination with Rc<T>. Recall that Rc<T> lets you have multiple owners of some data, but it only gives immutable access to that data. If you have an Rc<T> that holds a RefCell<T>, you can get a value that can have multiple owners and that you can mutate!.

read the examples in book.

---

## Reference Cycles Can Leak Memory

Rust’s memory safety guarantees make it difficult, but not impossible, to accidentally create memory that is never cleaned up (known as a memory leak). Preventing memory leaks entirely is not one of Rust’s guarantees, meaning memory leaks are memory safe in Rust. We can see that Rust allows memory leaks by using Rc<T> and RefCell<T>: It’s possible to create references where items refer to each other in a cycle. This creates memory leaks because the reference count of each item in the cycle will never reach 0, and the values will never be dropped.

### Preventing Reference Cycles Using `Weak<T>`

So far, we’ve demonstrated that calling `Rc::clone` increases the `strong_count` of an `Rc<T>` instance, and an `Rc<T>` instance is only cleaned up if its `strong_count` is 0. You can also create a weak reference to the value within an `Rc<T>` instance by calling `Rc::downgrade` and passing a reference to the `Rc<T>`. _Strong references_ are how you can share ownership of an `Rc<T>` instance. _Weak references_ don’t express an ownership relationship, and their count doesn’t affect when an `Rc<T>` instance is cleaned up. They won’t cause a reference cycle, because any cycle involving some weak references will be broken once the strong reference count of values involved is 0.

When you call `Rc::downgrade`, you get a smart pointer of type `Weak<T>`. Instead of increasing the `strong_count` in the `Rc<T>` instance by 1, calling `Rc::downgrade` increases the `weak_count` by 1. The `Rc<T>` type uses `weak_count` to keep track of how many `Weak<T>` references exist, similar to `strong_count`. The difference is the `weak_count` doesn’t need to be 0 for the `Rc<T>` instance to be cleaned up.

Because the value that `Weak<T>` references might have been dropped, to do anything with the value that a `Weak<T>` is pointing to you must make sure the value still exists. Do this by calling the `upgrade` method on a `Weak<T>` instance, which will return an `Option<Rc<T>>`. You’ll get a result of `Some` if the `Rc<T>` value has not been dropped yet and a result of `None` if the `Rc<T>` value has been dropped. Because `upgrade` returns an `Option<Rc<T>>`, Rust will ensure that the `Some` case and the `None` case are handled, and there won’t be an invalid pointer.
