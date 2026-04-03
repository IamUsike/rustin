## 5. Using Structs to Structure Related Data

    5.1 Defining and Instantiating Structs
    5.2 An example program using structs
    5.3 Methods

---

### 5.1 Defining and instantiating structs

- Similar to tuple type: Can hold multiple related values.
- unlike tuples, we name each piece of data. This makes `structs` more flexible than tuples. (Dont need to rely on the order of data to access the vaules of an instance)
- defined using `struct` keyword and data and types are defined within curly braces.

```rust
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}
```

- to use a struct after we've defined it, we need to create an _instance_ of it.
- the `key:value` specification can be done in any order

```rust
fn main() {
    let user1 = User {
        active: true,
        username: String::from("someusername123"),
        email: String::from("someone@example.com"),
        sign_in_count: 1,
    };
}
```

- to get a specific value from a struct, we use dot notation
- `user1.email`

- if a struct is _mutable_, the fields can even be edited

```rust
fn main() {
    let mut user1 = User {
        active: true,
        username: String::from("someusername123"),
        email: String::from("someone@example.com"),
        sign_in_count: 1,
    };

    user1.email = String::from("anotheremail@example.com");
}
```

- the whole struct needs to mutable, not individual elements.
- returning structs from fns

```rust
fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username: username,
        email: email,
        sign_in_count: 1,
    }
}
```

#### Using the field init shorthand

- since the fn params and body are same we can do this.

```rust
fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username,
        email,
        sign_in_count: 1,
    }
}
```

#### Creating Instances with struct update syntax

It’s often useful to create a new instance of a struct that includes most of the values from another instance of the same type, but changes some of them. You can do this using struct update syntax.

- `..` specifies that the fields that arent explicitly set should have the same value as the fields in the given instance.

```rust
fn main() {
    // --snip--

    let user2 = User {
        email: String::from("another@example.com"),
        ..user1
    };
}
```

> Note that the struct update syntax uses = like an assignment; this is because it moves the data, just as we saw in the “Variables and Data Interacting with Move” section. In this example, we can no longer use user1 after creating user2 because the String in the username field of user1 was moved into user2. If we had given user2 new String values for both email and username, and thus only used the active and sign_in_count values from user1, then user1 would still be valid after creating user2. Both active and sign_in_count are types that implement the Copy trait, so the behavior we discussed in the “Stack-Only Data: Copy” section would apply. We can also still use user1.email in this example, because its value was not moved out of user1.

#### Creating Different Types with Tuple Structs

Rust also supports structs that look similar to tuples, called tuple structs. Tuple structs have the added meaning the struct name provides but don’t have names associated with their fields; rather, they just have the types of the fields. Tuple structs are useful when you want to give the whole tuple a name and make the tuple a different type from other tuples, and when naming each field as in a regular struct would be verbose or redundant.

```rust
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

fn main() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);
}
```

#### Defining Unit-Like Structs

- We can define structs that dont have any fields.
- unit-like structs can be useful when implementing a trait on sometype but
  donot have data that you want to store in the type itself.

```rust
//declaring and instantiating unit-like struct
struct AlwaysEqual;

fn main() {
    let subject = AlwaysEqual;
}
```

---

### 5.3 Methods

Methods are similar to functions: We declare them with the fn keyword and a name, they can have parameters and a return value, and they contain some code that’s run when the method is called from somewhere else. Unlike functions, methods are defined within the context of a struct (or an enum or a trait object, which we cover in Chapter 6 and Chapter 18, respectively), and their first parameter is always self, which represents the instance of the struct the method is being called on.

#### Method syntax

```rust
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };

    println!(
        "The area of the rectangle is {} square pixels.",
        rect1.area()
    );
}
```

- To define the function within the context of Rectangle, we start an impl (implementation) block for Rectangle. Everything within this impl block will be associated with the Rectangle type.

> In the signature for area, we use &self instead of rectangle: &Rectangle. The &self is actually short for self: &Self. Within an impl block, the type Self is an alias for the type that the impl block is for. Methods must have a parameter named self of type Self for their first parameter, so Rust lets you abbreviate this with only the name self in the first parameter spot. Note that we still need to use the & in front of the self shorthand to indicate that this method borrows the Self instance, just as we did in rectangle: &Rectangle. Methods can take ownership of self, borrow self immutably, as we’ve done here, or borrow self mutably, just as they can any other parameter.

- we take `&self` cos we only want to read the values. If we wanted to change anything, we would've done `&mut self` as the first param.
- Having a method that takes ownership of the instance by using just self as the first parameter is rare; this technique is usually used when the method transforms self into something else and you want to prevent the caller from using the original instance after the transformation.

- **The main reason for using methods instead of functions, in addition to providing method syntax and not having to repeat the type of self in every method’s signature, is for organization.**

(there's a bit about getters, more about em in chapter7)

##### Where’s the `->` Operator?

In C and C++, two different operators are used for calling methods:

- Use `.` if you’re calling a method on the object directly
- Use `->` if you’re calling the method on a pointer to the object and need to dereference the pointer first

In other words, if `object` is a pointer:

```c
object->something()
```

is similar to:

```rust
(*object).something()
```

###### Rust's Approach

Rust doesn’t have an equivalent to the -> operator.
Instead, it uses a feature called automatic referencing and dereferencing.

Calling methods is one of the few places in Rust where this happens.

###### How it works

When you call a method like:

```rust
object.something()
```

Rust automatically adds &, &mut, or \* so that object matches the method’s expected signature.

For example, the following are equivalent:

```rust
p1.distance(&p2);
(&p1).distance(&p2);
```

The first one is just cleaner and more idiomatic.

#### Methods with More Parameters

Let’s practice using methods by implementing a second method on the Rectangle struct. This time we want an instance of Rectangle to take another instance of Rectangle and return true if the second Rectangle can fit completely within self (the first Rectangle); otherwise, it should return false.

```rust
fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };
    let rect2 = Rectangle {
        width: 10,
        height: 40,
    };
    let rect3 = Rectangle {
        width: 60,
        height: 45,
    };

    println!("Can rect1 hold rect2? {}", rect1.can_hold(&rect2));
    println!("Can rect1 hold rect3? {}", rect1.can_hold(&rect3));
}
```

- The expected output would look like the following because both dimensions of rect2 are smaller than the dimensions of rect1, but rect3 is wider than rect1:

```
Can rect1 hold rect2? true
Can rect1 hold rect3? false
```

- the can hold method

```rust
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}
```

#### Associated Functions

All functions defined within an impl block are called associated functions because they’re associated with the type named after the impl. We can define associated functions that don’t have self as their first parameter (and thus are not methods) because they don’t need an instance of the type to work with. We’ve already used one function like this: the String::from function that’s defined on the String type.

Associated functions that aren’t methods are often used for constructors that will return a new instance of the struct. These are often called new, but new isn’t a special name and isn’t built into the language. For example, we could choose to provide an associated function named square that would have one dimension parameter and use that as both width and height, thus making it easier to create a square Rectangle rather than having to specify the same value twice:

```rust
impl Rectangle {
    fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}
```

The Self keywords in the return type and in the body of the function are aliases for the type that appears after the impl keyword, which in this case is Rectangle.

To call this associated function, we use the :: syntax with the struct name; let sq = Rectangle::square(3); is an example. This function is namespaced by the struct: The :: syntax is used for both associated functions and namespaces created by modules.

#### Multiple impl Blocks

Each struct is allowed to have multiple impl blocks.

```rust
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

impl Rectangle {
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}
```

There’s no reason to separate these methods into multiple impl blocks here, but this is valid syntax.
