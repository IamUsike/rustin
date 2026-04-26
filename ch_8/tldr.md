## 8. Common Collections

8.1 Storing lists of values with vectors
8.2 Storing UTF-8 encoded text with strings
8.3 Storing keys with associated values in hashmaps

- these are mutable

---

### 8.1 Storing lists of values with vectors

- Can only store values of the same type.

#### Creating a vector

```rust
    let v: Vec<i32> = Vec::new();
```

- vectors are implemented from generics
- infers the type from the values (if given)

- Rust conveniently provides the vec! macro, which will create a new vector that holds the values you give it.

- The integer type is i32 because that’s the default integer type, as we discussed in the “Data Types” section of Chapter 3.

```rust
    let v = vec![1, 2, 3];
```

#### Updating a vector

- we need to make it mutable with the `mut` keyword

```rust
    let mut v = Vec::new();

    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);
```

#### Reading elements of vectors

There are two ways to reference a value stored in a vector: via indexing or by using the `get` method.

```rust
    let v = vec![1, 2, 3, 4, 5];

    let third: &i32 = &v[2];
    println!("The third element is {third}");

    let third: Option<&i32> = v.get(2);
    match third {
        Some(third) => println!("The third element is {third}"),
        None => println!("There is no third element."),
    }
```

- accessing out of bounds elements

```
    let v = vec![1, 2, 3, 4, 5];

    let does_not_exist = &v[100]; //program panicks
    let does_not_exist = v.get(100); //returns none
```

> imp
>
> > When the program has a valid reference, the borrow checker enforces the ownership and borrowing rules (covered in Chapter 4) to ensure that this reference and any other references to the contents of the vector remain valid. Recall the rule that states you can’t have mutable and immutable references in the same scope. That rule applies in Listing 8-6, where we hold an immutable reference to the first element in a vector and try to add an element to the end. This program won’t work if we also try to refer to that element later in the function.

```rust
    let mut v = vec![1, 2, 3, 4, 5];

    let first = &v[0];

    v.push(6);

    println!("The first element is: {first}");
```

- it will result in an error

```shell
$ cargo run
   Compiling collections v0.1.0 (file:///projects/collections)
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> src/main.rs:6:5
  |
4 |     let first = &v[0];
  |                  - immutable borrow occurs here
5 |
6 |     v.push(6);
  |     ^^^^^^^^^ mutable borrow occurs here
7 |
8 |     println!("The first element is: {first}");
  |                                      ----- immutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
error: could not compile `collections` (bin "collections") due to 1 previous error
```

#### Iterating Over the Values in a Vector

To access each element in a vector in turn, we would iterate through all of the elements rather than use indices to access one at a time. Listing 8-7 shows how to use a for loop to get immutable references to each element in a vector of i32 values and print them.

```rust
    let v = vec![100, 32, 57];
    for i in &v {
        println!("{i}");
    }
```

We can also iterate over mutable references to each element in a mutable vector in order to make changes to all the elements. The for loop in Listing 8-8 will add 50 to each element.

```rust
    let mut v = vec![100, 32, 57];
    for i in &mut v {
        *i += 50;
    }
```

#### Using an Enum to Store Multiple Types

Vectors can only store values that are of the same type. This can be inconvenient; there are definitely use cases for needing to store a list of items of different types. Fortunately, the variants of an enum are defined under the same enum type, so when we need one type to represent elements of different types, we can define and use an enum!.

```rust
    enum SpreadsheetCell {
        Int(i32),
        Float(f64),
        Text(String),
    }

    let row = vec![
        SpreadsheetCell::Int(3),
        SpreadsheetCell::Text(String::from("blue")),
        SpreadsheetCell::Float(10.12),
    ];
```

Rust needs to know what types will be in the vector at compile time so that it knows exactly how much memory on the heap will be needed to store each element. We must also be explicit about what types are allowed in this vector. If Rust allowed a vector to hold any type, there would be a chance that one or more of the types would cause errors with the operations performed on the elements of the vector. Using an enum plus a match expression means that Rust will ensure at compile time that every possible case is handled, as discussed in Chapter 6.

#### Dropping a Vector Drops its Elements

Like any other `struct`, a vector is freed when it goes out of scope

```rust
    {
        let v = vec![1, 2, 3, 4];

        // do stuff with v
    } // <- v goes out of scope and is freed here
```

---

### Storing UTF-8 Encoded Text with Strings

We discuss strings in the context of collections because strings are implemented as a collection of bytes, plus some methods to provide useful functionality when those bytes are interpreted as text.

#### Defining Strings

- Rust has only one string type defined in the core language: `str`, the string slice. Usually seen in its borrowed form `&str`
- string slices are references to some UTF-8 encoded string type.
- String literals are stored in the program's binary and are therefore string literals.

**The String type, which is provided by Rust’s standard library rather than coded into the core language, is a growable, mutable, owned, UTF-8 encoded string type.**

#### Creating a New String

Many of the same operations available with Vec<T> are available with String as well because String is actually implemented as a wrapper around a vector of bytes with some extra guarantees, restrictions, and capabilities.

```rust
let mut s = String::new()
```

- this line creates a new empty instance of a `string` called s.
- to have some initial data from which we want to start the string from, we can use any of the following methods.

```rust
    let data = "initial contents";

    let s = data.to_string();

    // The method also works on a literal directly:
    let s = "initial contents".to_string();
```

```rust
    let s = String::from("initial contents");
```

#### Updating a String

###### Appending with `push_str` or `push`

```rust
    let mut s = String::from("foo");
    s.push_str("bar");
```

- after these 2 lines `s` will contain `foobar`.
- the `push_str` method takes a string slice because we _dont necessarily want to take ownership of the param_.

- For eg, in the below snippet `s2` can still be used.

```rust
    let mut s1 = String::from("foo");
    let s2 = "bar";
    s1.push_str(s2);
    println!("s2 is {s2}");
```

- The `push` method takes a single character as a parameter and adds it to the String.

```rust
    let mut s = String::from("lo");
    s.push('l');
```

###### Concatenating with + or `format!`

```rust
    let s1 = String::from("Hello, ");
    let s2 = String::from("world!");
    let s3 = s1 + &s2; // note s1 has been moved here and can no longer be used
```

> -> `&str` = `&s[..]` <br> -> `&String` = `&s`

- If we need to concatenate multiple strings, the behavior of the `+` operator gets unwieldy:

```
    let s1 = String::from("tic");
    let s2 = String::from("tac");
    let s3 = String::from("toe");

    let s = s1 + "-" + &s2 + "-" + &s3;
```

- At this point, s will be tic-tac-toe. With all of the + and " characters, it’s difficult to see what’s going on. For combining strings in more complicated ways, we can instead use the format! macro:

```rust
fn main() {
    let s1 = String::from("tic");
    let s2 = String::from("tac");
    let s3 = String::from("toe");

    let s = format!("{s1}-{s2}-{s3}");
}
```

format works similar to println (borrows the strings) but instead of printing it returns `s`

#### indexing into Strings

```rust
    let s1 = String::from("hi");
    let h = s1[0];
```

- this throws an error.
  **RUST DOESNOT IMPLEMENT INDEXING FOR STRINGS**
  But why ? (you'll have to read the book for that!)

#### Slicing Strings

```rust
let hello = "Здравствуйте";

let s = &hello[0..4];
```

Here, s will be a &str that contains the first 4 bytes of the string. Earlier, we mentioned that each of these characters was 2 bytes, which means s will be Зд.

If we were to try to slice only part of a character’s bytes with something like &hello[0..1], Rust would panic at runtime in the same way as if an invalid index were accessed in a vector:

```rust
$ cargo run
   Compiling collections v0.1.0 (file:///projects/collections)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s
     Running `target/debug/collections`

thread 'main' panicked at src/main.rs:4:19:
byte index 1 is not a char boundary; it is inside 'З' (bytes 0..2) of `Здравствуйте`
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

#### Iterating Over Strings

The best way to operate on pieces of strings is to be explicit about whether you want characters or bytes. For individual Unicode scalar values, use the chars method. Calling chars on “Зд” separates out and returns two values of type char, and you can iterate over the result to access each element:

```rust
for c in "Зд".chars() {
    println!("{c}");
}
```

The code will print the following:

```
З
д
```

Alternatively, the bytes method returns each raw byte, which might be appropriate for your domain:

```rust
for b in "Зд".bytes() {
    println!("{b}");
}
```

the code will print the string

```rust
208
151
208
180
```

But be sure to remember that valid Unicode scalar values may be made up of more than 1 byte.

---

### Storing Keys with Associated Values in Hash Maps

The type `HashMap<K, V>` stores a mapping of keys of type `K` to values of type `V` using a hashing function, which determines how it places these keys and values into memory. (as always check the standard lib documentation)

#### Creating a New Hash Map

```rust
    use std::collections::HashMap;

    let mut scores = HashMap::new();

    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);
```

Just like vectors, hash maps store their data on the heap. This HashMap has keys of type String and values of type i32. Like vectors, hash maps are homogeneous: All of the keys must have the same type, and all of the values must have the same type.

#### Accessing Values in a Hash Map

```rust
    use std::collections::HashMap;

    let mut scores = HashMap::new();

    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);

    let team_name = String::from("Blue");
    let score = scores.get(&team_name).copied().unwrap_or(0);
```

Here, score will have the value that’s associated with the Blue team, and the result will be 10. The get method returns an `Option<&V>`; if there’s no value for that key in the hash map, get will return None. This program handles the Option by calling copied to get an `Option<i32>` rather than an `Option<&i32>`, then `unwrap_or` to set score to zero if scores doesn’t have an entry for the key.

We can iterate over each key-value pair in a hash map in a similar manner as we do with vectors, using a for loop:

```rust
    use std::collections::HashMap;

    let mut scores = HashMap::new();

    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);

    for (key, value) in &scores {
        println!("{key}: {value}");
    }
```

this will print each pair in an arbitrary order.

#### Managing Ownership in Hash Maps

For types that implement the Copy trait, like i32, the values are copied into the hash map. For owned values like String, the values will be moved and the hash map will be the owner of those values.

```rust
    use std::collections::HashMap;

    let field_name = String::from("Favorite color");
    let field_value = String::from("Blue");

    let mut map = HashMap::new();
    map.insert(field_name, field_value);
    // field_name and field_value are invalid at this point, try using them and
    // see what compiler error you get!
```

If we insert references to values into the hash map, the values won’t be moved into the hash map. The values that the references point to must be valid for at least as long as the hash map is valid. We’ll talk more about these issues in “Validating References with Lifetimes” in Chapter 10.

#### Updating a hashmap

Each unique key can only have one value associated with it at a time.

When you want to change the data in a hash map, you have to decide how to handle the case when a key already has a value assigned. You could replace the old value with the new value, completely disregarding the old value. You could keep the old value and ignore the new value, only adding the new value if the key doesn’t already have a value. Or you could combine the old value and the new value. Let’s look at how to do each of these!

###### Overwriting a value

If we insert a key and a value into a hash map and then insert that same key with a different value, the value associated with that key will be replaced.

```rust
    use std::collections::HashMap;

    let mut scores = HashMap::new();

    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Blue"), 25);

    println!("{scores:?}"); #25
```

###### Adding a key and value only if a key isnt present

```rust
    use std::collections::HashMap;

    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);

    scores.entry(String::from("Yellow")).or_insert(50);
    scores.entry(String::from("Blue")).or_insert(50);

    println!("{scores:?}");
```

###### updating a value based on the old value

```rust
    use std::collections::HashMap;

    let text = "hello world wonderful world";

    let mut map = HashMap::new();

    for word in text.split_whitespace() {
        let count = map.entry(word).or_insert(0);
        *count += 1;
    }

    println!("{map:?}");
```
