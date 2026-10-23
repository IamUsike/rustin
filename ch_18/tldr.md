# Object Oriented Programming Features

## Characteristics of Object-Oriented Languages

- object-oriented programs are made up of objects. An Object packages both data and the procedures
  that operate on that data. The procedures area typically called methods or operations.

> Using this definition, Rust is object oriented: Structs and enums have data, and impl blocks provide methods on structs and enums. Even though structs and enums with methods aren’t called objects, they provide the same functionality, according to the Gang of Four’s definition of objects.

### Encapsulation That Hides Implementation Details

- `pub` structs => struct is accessible outside the module but unless the
  individual fields are pub, they cant be mod-ed directly. Can only be done
  using api's. (the public methods on that struct)

> If encapsulation is a required aspect for a language to be considered object oriented, then Rust meets that requirement. The option to use pub or not for different parts of code enables encapsulation of implementation details.

### Inheritance as a Type System and as Code Sharing

- Inheritance is a mechanism whereby an object can inherit elements from another object’s definition, thus gaining the parent object’s data and behavior without you having to define them again.

- rust cant do this directly.

**There are 2 main reasons to choose inheritance**

1. reuse of code.

- we can do this in a limited way using traits (defualt impl).
- we can also override this default impl.

2. To enable the child type to be used in the same places as the parent type.
   This is also called polymorphism. (we can substitute multiple objects for
   each other at runtime if they share certain Characteristics).

Rust has chosen a different set of trade-offs by not offering inheritance. Inheritance is often at risk of sharing more code than necessary. Subclasses shouldn’t always share all characteristics of their parent class but will do so with inheritance. This can make a program’s design less flexible. It also introduces the possibility of calling methods on subclasses that don’t make sense or that cause errors because the methods don’t apply to the subclass. In addition, some languages will only allow single inheritance (meaning a subclass can only inherit from one class), further restricting the flexibility of a program’s design.

For these reasons, Rust takes the different approach of using trait objects instead of inheritance to achieve polymorphism at runtime. Let’s look at how trait objects work.

---

## Using Trait Objects to abstract over Shared Behavior

(read trpl for more from here. very brief :shrug)

## Using Trait Objects to Abstract over shared behavior

### Defining a Trait for Common Behavior

- **Trait object**: Points to both an instance of a type implementing our specified trait and a table used to look up trait methods on that type at runtime.
- We create a trait object by specifying some sort of pointer, such as a reference or a smart pointer `Box<T>`, and then using the `dyn` keyword and then specifying the relevant trait.
- Trait objects can be used in the place of a generic or a concrete type.
- Wherever we use a trait object, Rust's type system will ensure at compile time that any value used in that context will implement the trait object's trait. Consequently, we don't need to know all the possible types at compile time.

```rust
fn main() {
    println!("Hello, Rust!");
}

pub trait Draw {
    fn draw(&self);
}

// Box<dyn Draw> is a stand-in for any type that implements Draw
pub struct Screen {
    pub components: Vec<Box<dyn Draw>>,
}

// This differs from defining a struct that uses a generic type param
// with trait bounds. A generic type param can be substituted with only
// one concrete type param at a time, whereas trait objects allow for
// multiple concrete types to fill in for the trait obj at runtime.

impl Screen {
    fn run(&self) {
        for component in self.components.iter() {
            component.draw();
        }
    }
}
```

### Implementing the Trait

Some more code

### Performing Dynamic Dispatch

- Generics use **static dispatch**: the compiler will know the method we are calling during compile time (**monomorphization**).
- In **dynamic dispatch**, the compiler doesn't know the methods during compile time. Instead, it is looked up using a **vtable** at runtime.

////////////////////////////////////////////////////////////////////////////////////////////////////

## Implementing an Object-Oriented Design Pattern

> **State pattern** is an object-oriented design pattern.

- The crux of this is we define a set of states a value can have internally.
- The states are represented by a set of state objects, and the value's **behavior changes based on its state**.
- The state objects share functionality (traits and structs).
- Each state is responsible for its own state and when it should transition to another state.
