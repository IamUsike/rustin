## 7. Packages, Crates and Module

- 7.1 Packages and Crates
- 7.2 Control Scope and privacy scope with modules
- 7.3 Paths for referring to an item in the module tree
- 7.4 Bringing Paths into scope with the use keyword
- 7.5 Separating Modules into different Files

---

### 7.1 Packages and Crates

- as project grows, the code should be organized by splitting it into multiple modules and files.
- A package can contain multiple binary crates and optionally one library crate.

> For very large projects comprising a set of interrelated packages that evolve together, Cargo provides workspaces, which we’ll cover in “Cargo Workspaces” in Chapter 14.

Rust has a number of features that allow you to manage your code’s organization, including which details are exposed, which details are private, and what names are in each scope in your programs. These features, sometimes collectively referred to as the module system, include:

- Packages: A Cargo feature that lets you build, test, and share crates
- Crates: A tree of modules that produces a library or executable
- Modules and use: Let you control the organization, scope, and privacy of paths
- Paths: A way of naming an item, such as a struct, function, or module

---

- A _crate_ is the smallest amount of code that the rust compiler considers at a time.
- There's 2 types of crates
  - **Binary Crates**: Programs that can be compiled into an executable and run. Eg: Server, CLI program. Must have the `main` fn.
  - **Library Crates**: Dont have `main` fn and they dont compile into an executable. These define functionality intended to be shared with multiple projects.
- The crate root is a source file that the Rust compiler starts from and makes up the root module of your crate (we’ll explain modules in depth in “Control Scope and Privacy with Modules”).

- A package is a bundle of one or more crates that provides a set of functionality. A package contains a Cargo.toml file that describes how to build those crates.

> A package is a bundle of one or more crates that provides a set of functionality. A package contains a Cargo.toml file that describes how to build those crates.

- If a package contains src/main.rs and src/lib.rs, it has two crates: a binary and a library, both with the same name as the package. (cargo defaults)
- A package can have multiple binary crates by placing files in the src/bin directory: Each file will be a separate binary crate.

---

### 7.2 Control Scope and Privacy with Modules

#### Module Cheat Sheet

- **Start from the crate root**: When compiling a crate, the compiler first looks in the crate root file (usually src/lib.rs for a library crate and src/main.rs for a binary crate) for code to compile.
- **Declaring modules**: In the crate root file, you can declare new modules; say you declare a “garden” module with `mod garden;`. The compiler will look for the module’s code in these places:
  - Inline, within curly brackets that replace the semicolon following `mod garden`
  - In the file src/garden.rs
  - In the file src/garden/mod.rs
- **Paths to code in modules:** Once a module is part of your crate, you can refer to code in that module from anywhere else in that same crate, as long as the privacy rules allow, using the path to the code. For example, an Asparagus type in the garden vegetables module would be found at `crate::garden::vegetables::Asparagus`.
- **Private vs. public:** Code within a module is private from its parent modules by default. To make a module public, declare it with `pub mod` instead of `mod`. To make items within a public module public as well, use `pub` before their declarations.
- **The `use` keyword**: Within a scope, the `use` keyword creates shortcuts to items to reduce repetition of long paths. In any scope that can refer to `crate::garden::vegetables::Asparagus`, you can create a shortcut with use `crate::garden::vegetables::Asparagus;`, and from then on you only need to write `Asparagus` to make use of that type in the scope.

#### Grouping Related Code in modules

- Modules let us organize code within a crate for readability and easy reuse.
- allow us to control the privacy of the items.

---

### 7.3 Paths for Referring to an Item in the Module Tree

- To show Rust where to find an item in a module tree, we use a path in the same way we use a path when navigating a filesystem.

A path can take 2 forms

- Absolute path: starts with the literal `crate`. (from root).
- Relative path: starts from the current module and uses `self`, `super` or an identifier in the
  current module.

Both absolute and relative paths are followed by one or more identifiers separated by double colons (::).

- In Rust, all items (functions, methods, structs, enums, modules, and constants) are private to parent modules by default. If you want to make an item like a function or struct private, you put it in a module.

- Items in a parent module can’t use the private items inside child modules, but items in child modules can use the items in their ancestor modules. This is because child modules wrap and hide their implementation details, but the child modules can see the context in which they’re defined. (can be made available to the parent module by using the pub keyword)

#### Exposing Paths with the `pub` keyword

```rust
mod front_of_house {
    pub mod hosting {
        fn add_to_waitlist() {}
    }
}

// -- snip --
```

This still throws error (ps: The prev example didnt have `mod hosting` as `pub`)

```
$ cargo build
   Compiling restaurant v0.1.0 (file:///projects/restaurant)
error[E0603]: function `add_to_waitlist` is private
  --> src/lib.rs:10:37
   |
10 |     crate::front_of_house::hosting::add_to_waitlist();
   |                                     ^^^^^^^^^^^^^^^ private function
   |
note: the function `add_to_waitlist` is defined here
  --> src/lib.rs:3:9
   |
 3 |         fn add_to_waitlist() {}
   |         ^^^^^^^^^^^^^^^^^^^^

error[E0603]: function `add_to_waitlist` is private
  --> src/lib.rs:13:30
   |
13 |     front_of_house::hosting::add_to_waitlist();
   |                              ^^^^^^^^^^^^^^^ private function
   |
note: the function `add_to_waitlist` is defined here
  --> src/lib.rs:3:9
   |
 3 |         fn add_to_waitlist() {}
   |         ^^^^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0603`.
error: could not compile `restaurant` (lib) due to 2 previous errors

```

- just make the `fn` public with the `pub` keyword.

#### Starting Relative Paths with `super`

- We can construct relative paths that begin in the parent module, rather than the current module or the crate root, by using super at the start of the path.
- Using super allows us to reference an item that we know is in the parent module.

```rust
fn deliver_order() {}

mod back_of_house {
    fn fix_incorrect_order() {
        cook_order();
        super::deliver_order();
    }

    fn cook_order() {}
}
```

- The fix_incorrect_order function is in the back_of_house module, so we can use super to go to the parent module of back_of_house, which in this case is crate, the root. From there, we look for deliver_order and find it. Success! We think the back_of_house module and the deliver_order function are likely to stay in the same relationship to each other and get moved together should we decide to reorganize the crate’s module tree. Therefore, we used super so that we’ll have fewer places to update code in the future if this code gets moved to a different module.

#### Making Structs and Enums public

- structs and enums can also be made public.
- if we use `pub` on a struct, the struct itslef will be public but the fields wont.
- we need to make individual fields also public.

```rust
mod back_of_house {
    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String,
    }

    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("peaches"),
            }
        }
    }
}

pub fn eat_at_restaurant() {
    // Order a breakfast in the summer with Rye toast.
    let mut meal = back_of_house::Breakfast::summer("Rye");
    // Change our mind about what bread we'd like.
    meal.toast = String::from("Wheat");
    println!("I'd like {} toast please", meal.toast);

    // The next line won't compile if we uncomment it; we're not allowed
    // to see or modify the seasonal fruit that comes with the meal.
    // meal.seasonal_fruit = String::from("blueberries");
}
```

- if we make an enum public, all the fields will be public too.

```rust
mod back_of_house {
    pub enum Appetizer {
        Soup,
        Salad,
    }
}

pub fn eat_at_restaurant() {
    let order1 = back_of_house::Appetizer::Soup;
    let order2 = back_of_house::Appetizer::Salad;
}
```

---

### Bringing Paths into Scope with the `use` keyword

- instead of using absolute or relative paths always to call functions, We can create a shortcut to a path with the use keyword once and then use the shorter name everywhere else in the scope.

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
}
```

###### suuuuuuuper (franky)

```rust
mod outer {
    pub fn a() {}

    mod inner {
        pub fn b() {
            super::a(); // works
        }
    }
}
```

#### Providing New names with the `as` keyword

```rust
use std::fmt::Result;
use std::io::Result as IoResult;

fn function1() -> Result {
    // --snip--
}

fn function2() -> IoResult<()> {
    // --snip--
}
```

#### Re-exporting Names with `pub use`

When we bring a name into scope with the use keyword, the name is private to the scope into which we imported it. To enable code outside that scope to refer to that name as if it had been defined in that scope, we can combine pub and use. This technique is called re-exporting because we’re bringing an item into scope but also making that item available for others to bring into their scope.

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

pub use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
}
```

#### using external packages

Members of the Rust community have made many packages available at crates.io, and pulling any of them into your package involves these same steps: listing them in your package’s Cargo.toml file and using use to bring items from their crates into scope.

#### using nested paths to clean up `use` lists

If we’re using multiple items defined in the same crate or same module, listing each item on its own line can take up a lot of vertical space in our files. For example, these two use statements we had in the guessing game in Listing 2-4 bring items from std into scope:

```rust
// --snip--
use std::{cmp::Ordering, io};
// --snip--
```

#### importing items with the Glob operator

- bring all the public items defined in a path to scope

```rust
use std::collections::*;
```

---

### Separating Modules into Different Files.

just read it from there bro. Its too hot here.
