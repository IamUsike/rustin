3. Common Programming Concepts<br>
   3.1. Variables and Mutability<br>
   3.2. Data Types<br>
   3.3. Functions<br>
   3.4. Comments<br>
   3.5. Control Flow<br>

---

### Variables and Mutability

- Variables are _immutable_ by default

```rust
fn main() {
    let x = 5;
    println!("The value of x is: {x}");
    x = 6;
    println!("The value of x is: {x}");
}
```

The above snippet will throw a _compile time_ error.

- variables can be made _mutable_ by using the `mut` keyword.

```rust
fn main() {
    let mut x = 5;
    println!("The value of x is: {x}");
    x = 6;
    println!("The value of x is: {x}");
}
```

##### Declaring Constants

- like immutable vars, const values are bound to their names and arent allowed to change.
- `mut` cant be used for constants (always immutable)
- Constants are declared using the `const` keyword.
- Data type of the constant **must** always be annotated.
- `let` vars cant be global but `const` can be.
- constants may be set only to a constant expression, not the result of a value that could only be computed at runtime.

eg:

```rust
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
```

> The compiler is able to evaluate a limited set of operations at compile time, which lets us choose to write out this value in a way that’s easier to understand and verify, rather than setting this constant to the value 10,800. See the [Rust Reference’s section on constant evaluation](https://doc.rust-lang.org/reference/const_eval.html) for more information on what operations can be used when declaring constants.

##### Shadowing

- declaring a new variable with the same name overrides the first declaration.
- the second var is said to have overshadowed the first.

```rust
fn main() {
    let x = 5;

    let x = x + 1;

    {
        let x = x * 2; //12
        println!("The value of x in the inner scope is: {x}");
    }

    println!("The value of x is: {x}"); //6
}
```

Shadowing is different from marking a variable as mut because we’ll get a compile-time error if we accidentally try to reassign to this variable without using the let keyword. By using let, we can perform a few transformations on a value but have the variable be immutable after those transformations have completed.

The other difference between mut and shadowing is that because we’re effectively creating a new variable when we use the let keyword again, we can change the type of the value but reuse the same name. _(we arent allowed to mutate a variables type)_

---

### Data Types

- every value in rust is of a certain data type(Scalar or Compound)

#### Scalar Types

- represents a single value.
- 4 primary scalar types
  - characters
  - integers
  - booleans
  - floating point

###### Integers

- number without a fractional component.

types of integers in rust
| Length | Signed | Unsigned |
|------|------|------|
| 8-bit | i8 | u8 |
| 16-bit | i16 | u16 |
| 32-bit | i32 | u32 |
| 64-bit | i64 | u64 |
| 128-bit | i128 | u128 |
| Architecture-dependent | isize | usize |

**integer overflow**
-> If compiled in `--debug` mode, the program panics.
-> In release mode `--release`, two's complement wrapping takes place.

To explicitly handle the possibility of overflow, you can use these families of methods provided by the standard library for primitive numeric types:

- Wrap in all modes with the `wrapping_*` methods, such as `wrapping_add`.
- Return the None value if there is overflow with the `checked_*` methods.
- Return the value and a Boolean indicating whether there was overflow with the `overflowing_*` methods.
- Saturate at the value’s minimum or maximum values with the `saturating_*` methods.

###### Floating-Point

- two primitive floating point types
  - decimals
  - numbers
- 2 types (f64 & f32)

```rust
fn main() {
    let x = 2.0; // f64

    let y: f32 = 3.0; // f32
}
```

**numeric ops**

```rust
fn main() {
    // addition
    let sum = 5 + 10;

    // subtraction
    let difference = 95.5 - 4.3;

    // multiplication
    let product = 4 * 30;

    // division
    let quotient = 56.7 / 32.2;
    let truncated = -5 / 3; // Results in -1

    // remainder
    let remainder = 43 % 5;
}
```

###### Boolean type

- 1 byte

```rust
fn main() {
    let t = true;

    let f: bool = false; // with explicit type annotation
}
```

###### Chatacter Type

- specified using single quotation marks
- 4bytes unicode scalar value

```rust
fn main() {
    let c = 'z';
    let z: char = 'ℤ'; // with explicit type annotation
    let heart_eyed_cat = '😻';
}
```

#### Compound Types

- Can group multiple primitive values into one
- 2 primitive compound types
  - tuple
  - array

##### The Tuple type

- each element can be of a different type.
- fixed in length. Cant grow or shrink in size post declaration.
- We create a tuple by writing a comma-separated list of values inside parentheses. Each position in the tuple has a type, and the types of the different values in the tuple don’t have to be the same.

```rust
fn main() {
    let tup: (i32, f64, u8) = (500, 6.4, 1);
}
```

- tup is considered a single compound element
- to get individual vals. Can do pattern matching (destructuring)

```rust
fn main() {
    let tup = (500, 6.4, 1);

    let (x, y, z) = tup;

    println!("The value of y is: {y}");
}
```

- elements can also be accessed using dot notation.

```rust
fn main() {
    let x: (i32, f64, u8) = (500, 6.4, 1);

    let five_hundred = x.0;

    let six_point_four = x.1;

    let one = x.2;
}
```

> The tuple without any values has a special name, unit. This value and its corresponding type are both written () and represent an empty value or an empty return type. Expressions implicitly return the unit value if they don’t return any other value.

#### The array type

- unlike tuple each element needs to be of the same type.
- have a fixed length.

```rust
fn main() {
    let a = [1, 2, 3, 4, 5];
}
```

- all primitive types are stored on the stack.(incl array and tup)

- You write an array’s type using square brackets with the type of each element, a semicolon, and then the number of elements in the array, like so:
  let a: [i32; 5] = [1, 2, 3, 4, 5];

- initialize an array that contains 5 elements (3)
  `let a = [3; 5];
`
  **Array Element Access**

```rust
fn main() {
    let a = [1, 2, 3, 4, 5];

    let first = a[0];
    let second = a[1];
}
```

> Accessing out of bound elements causes the program to panic.

---

### Functions

```rust
fn main() {
    println!("Hello, world!");

    another_function();
}

fn another_function() {
    println!("Another function.");
}
```

- Rust doesn’t care where you define your functions, only that they’re defined somewhere in a scope that can be seen by the caller.

#### Parameters

```rust
fn main() {
    another_function(5);
}

fn another_function(x: i32) {
    println!("The value of x is: {x}");
}
```

- the type of Parameters must always be annotated in function signatures

#### Statements and expressions

_rust is an expression based language_

- Statements are instructions that perform some action and do not return a value.
- Expressions evaluate to a resultant value.

statements:

- `let y = 6`
- function definitions (calling a function is not statement)

- Expressions evaluate to a value and make up most of the rest of the code that you’ll write in Rust. Consider a math operation, such as 5 + 6, which is an expression that evaluates to the value 11. Expressions can be part of statements: In Listing 3-1, the 6 in the statement let y = 6; is an expression that evaluates to the value 6. Calling a function is an expression. Calling a macro is an expression. A new scope block created with curly brackets is an expression

```rust
fn main() {
    let y = {
        let x = 3;
        x + 1
    };

    println!("The value of y is: {y}");
}
```

is a block that, in this case, evaluates to 4. That value gets bound to y as part of the let statement. Note the x + 1 line without a semicolon at the end, which is unlike most of the lines you’ve seen so far. _Expressions do not include ending semicolons. If you add a semicolon to the end of an expression, you turn it into a statement, and it will then not return a value._ Keep this in mind as you explore function return values and expressions next.

##### Functions with return values

- need to specify the return type using `->`
- can return by mentioning `return` keyword, or return implicitly at the end by skipping the semicolon.

```rust
fn five() -> i32 {
    5
}

fn main() {
    let x = five();

    println!("The value of x is: {x}");
}
```

---

### Control Flow

#### If statements

```rust
fn main() {
    let number = 3;

    if number < 5 {
        println!("condition was true");
    } else { //can also use elseif
        println!("condition was false");
    }
}
```

##### Using if in a let Statement

- Because if is an expression, we can use it on the right side of a let statement to assign the outcome to a variable,

```rust
fn main() {
    let condition = true;
    let number = if condition { 5 } else { 6 }; //values in branches must be of the same type

    println!("The value of number is: {number}");
}
```

#### Repeating with loops

- infinite loop

```rust
fn main() {
    loop {
        println!("again!");
    }
}
```

- can place break keyword to exit out of the loop

#### returning values from loop

- add the value that you want to be returned after the break statement.
- can also use the `return` keyword. But that'll exit out of the whole function.

```rust
fn main() {
    let mut counter = 0;

    let result = loop {
        counter += 1;

        if counter == 10 {
            break counter * 2;
        }
    };

    println!("The result is {result}");
}
```

##### Disambiguating with Loop Labels

If you have loops within loops, break and continue apply to the innermost loop at that point. You can optionally specify a loop label on a loop that you can then use with break or continue to specify that those keywords apply to the labeled loop instead of the innermost loop. Loop labels must begin with a single quote. Here’s an example with two nested loops:

```
fn main() {
    let mut count = 0;
    'counting_up: loop {
        println!("count = {count}");
        let mut remaining = 10;

        loop {
            println!("remaining = {remaining}");
            if remaining == 9 {
                break;
            }
            if count == 2 {
                break 'counting_up;
            }
            remaining -= 1;
        }

        count += 1;
    }
    println!("End count = {count}");
}
```

```output
$ cargo run
   Compiling loops v0.1.0 (file:///projects/loops)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.58s
     Running `target/debug/loops`
count = 0
remaining = 10
remaining = 9
count = 1
remaining = 10
remaining = 9
count = 2
remaining = 10
End count = 2
```

#### why so much hype for a while loop ?

```rust
fn main() {
    let mut number = 3;

    while number != 0 {
        println!("{number}!");

        number -= 1;
    }

    println!("LIFTOFF!!!");
}
```

##### looping through collections using for

```rust
fn main() {
    let a = [10, 20, 30, 40, 50];

    for element in a {
        println!("the value is: {element}");
    }
}
```

```rust
fn main() {
    for number in (1..4).rev() {
        println!("{number}!");
    }
    println!("LIFTOFF!!!");
}
```
