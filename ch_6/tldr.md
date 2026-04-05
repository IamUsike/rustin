## 6. Enums and Pattern Matching

    6.1. Defining an Enum
    6.2. The match Control Flow Construct
    6.3. Concise Control Flow with if let and let...else

---

### 6.1 Defining an Enum

Where structs give you a way of grouping together related fields and data, like a Rectangle with its width and height, enums give you a way of saying a value is one of a possible set of values.

```rust
enum IpAddrKind {
    V4,
    V6,
}
```

- IpAddrKind is now a custom data type that we can use elsewhere in our code.

#### Enum Values

- we can create instances of each of the enum variants like this:

```rust
    let four = IpAddrKind::V4;
    let six = IpAddrKind::V6;
```

- fn params

```rust
fn route(ip_kind: IpAddrKind) {}

    route(IpAddrKind::V4);
    route(IpAddrKind::V6);
```

- we can put raw data directly inside an enum variant.
- This new definition of the IpAddr enum says that both V4 and V6 variants will have associated String values:

```rust
    enum IpAddr {
        V4(String),
        V6(String),
    }

    let home = IpAddr::V4(String::from("127.0.0.1"));

    let loopback = IpAddr::V6(String::from("::1"));
```

- The name of each enum variant that we define also becomes a function that constructs an instance of the enum. That is, IpAddr::V4() is a function call that takes a String argument and returns an instance of the IpAddr type. We automatically get this constructor function defined as a result of defining the enum.

- Just as we’re able to define methods on structs using impl, we’re also able to define methods on enums. Here’s a method named call that we could define on our Message enum:

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

    impl Message {
        fn call(&self) {
            // method body would be defined here
        }
    }

    let m = Message::Write(String::from("hello"));
    m.call();
```

The body of the method would use self to get the value that we called the method on. In this example, we’ve created a variable m that has the value Message::Write(String::from("hello")), and that is what self will be in the body of the call method when m.call() runs.

#### The option Enum

- The Option type encodes the very common scenario in which a value could be something, or it could be nothing.

```rust
enum Option<T> {
    None,
    Some(T),
}
```

The Option<T> enum is so useful that it’s even included in the prelude; you don’t need to bring it into scope explicitly. Its variants are also included in the prelude: You can use Some and None directly without the Option:: prefix. The Option<T> enum is still just a regular enum, and Some(T) and None are still variants of type Option<T>.

The <T> syntax is a feature of Rust we haven’t talked about yet. It’s a generic type parameter, and we’ll cover generics in more detail in Chapter 10. For now, all you need to know is that <T> means that the Some variant of the Option enum can hold one piece of data of any type, and that each concrete type that gets used in place of T makes the overall Option<T> type a different type. Here are some examples of using Option values to hold number types and char types:

```rust
    let some_number = Some(5);
    let some_char = Some('e');

// i32 for 'some'
    let absent_number: Option<i32> = None;
```

- cant add `i8` to `option<i8>`

```rust
    let x: i8 = 5;
    let y: Option<i8> = Some(5);

    let sum = x + y;
```

---

### The `match` Control Flow Construct

Rust has an extremely powerful control flow construct called match that allows you to compare a value against a series of patterns and then execute code based on which pattern matches. Patterns can be made up of literal values, variable names, wildcards, and many other things;

```rust
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => {
          println("Lunckyy");
          1
        },
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}
```

#### Patterns that bind to values

Another useful feature of match arms is that they can bind to the parts of the values that match the pattern. This is how we can extract values out of enum variants.

```rust
#[derive(Debug)] // so we can inspect the state in a minute
enum UsState {
    Alabama,
    Alaska,
    // --snip--
}

enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(UsState),
}
```

In the match expression for this code, we add a variable called state to the pattern that matches values of the variant Coin::Quarter. When a Coin::Quarter matches, the state variable will bind to the value of that quarter’s state. Then, we can use state in the code for that arm, like so:

```rust
fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter(state) => {
            println!("State quarter from {state:?}!");
            25
        }
    }
}
```

If we were to call value_in_cents(Coin::Quarter(UsState::Alaska)), coin would be Coin::Quarter(UsState::Alaska). When we compare that value with each of the match arms, none of them match until we reach Coin::Quarter(state). At that point, the binding for state will be the value UsState::Alaska. We can then use that binding in the println! expression, thus getting the inner state value out of the Coin enum variant for Quarte

#### The Option<T> match Pattern

Let’s say we want to write a function that takes an Option<i32> and, if there’s a value inside, adds 1 to that value. If there isn’t a value inside, the function should return the None value and not attempt to perform any operations.

```rust
    fn plus_one(x: Option<i32>) -> Option<i32> {
        match x {
            None => None,
            Some(i) => Some(i + 1),
        }
    }

    let five = Some(5);
    let six = plus_one(five);
    let none = plus_one(None);
```

#### Matches are Exhaustive

There’s one other aspect of match we need to discuss: The arms’ patterns must cover all possibilities. Consider this version of our plus_one function, which has a bug and won’t compile:

```rust fn plus_one(x: Option<i32>) -> Option<i32> {
        match x {
            Some(i) => Some(i + 1),
        }
    }
//didnt handle the none case
```

#### Catch-All Patterns and the \_ Placeholder

Using enums, we can also take special actions for a few particular values, but for all other values take one default action.

```rust
    let dice_roll = 9;
    match dice_roll {
        3 => add_fancy_hat(),
        7 => remove_fancy_hat(),
        _ => reroll(),
    }

    fn add_fancy_hat() {}
    fn remove_fancy_hat() {}
    fn reroll() {}
```

---

### Concise Control Flow with `if let` and `let...else`

- instead of this

```rust
    let config_max = Some(3u8);
    match config_max {
        Some(max) => println!("The maximum is configured to be {max}"),
        _ => (),
    }
```

- do this

```rust
    let config_max = Some(3u8);
    if let Some(max) = config_max {
        println!("The maximum is configured to be {max}");
    }
```

some other shii sleepy to jot. gn
