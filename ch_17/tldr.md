# Fundamentals of Async Programming: Async, Await, Futures and Streams

- Async programming is an abstraction that lets us express our code in terms of potential pausing points and eventual results that takes care of the details of coordination for us.

-> **Parallelism and Concurrency**

- when an individual works on several dift tasks before any of them is complete is called _concurrency_
- when the team splits up a group of tasks and each member works on them independently is called _Parallelism_
- parallelism and concurrency can intersect with each other too.

---

## Futures and the Async Syntax

- A _future_ is a value that may not be ready now but will become ready at some point in the future.
- Rust provides a `Future` trait.
- futures are the types that implement the `Future` trait.
- each future holds its own info about the progress that has been made and what "ready" means.

You can apply the async keyword to blocks and functions to specify that they can be interrupted and resumed. Within an async block or async function, you can use the await keyword to await a future (that is, wait for it to become ready). Any point where you await a future within an async block or function is a potential spot for that block or function to pause and resume. The process of checking with a future to see if its value is available yet is called polling.

### Our first async program

- Futures are _lazy_. They dont do anything until we ask them to with the `await` keyword.
- When Rust sees a block marked with the async keyword, it compiles it into a unique, anonymous data type that implements the Future trait. When Rust sees a function marked with async, it compiles it into a non-async function whose body is an async block. An async function’s return type is the type of the anonymous data type the compiler creates for that async block.

(go through the book for code snippets and examples)

- main cannot be async cos async code needs a _runtime_: A rust crate that manages the details of executing async code.
- A program's main fn can initialize a runtime, but it's _not_ a runtime itself.

- Each _await point_ that is, every place where the code uses the `await` keyword - where the control is handed back to the runtime.
- to make this work, Rust needs to keep track of the state involved in the async block so that the runtime could kick off some other work and then come back when it's ready to try advancing once again.
- this is like an invisible state machine, analogous to this enum:

```rust
enum PageTitleFuture<'a> {
    Initial { url: &'a str },
    GetAwaitPoint { url: &'a str },
    TextAwaitPoint { response: trpl::Response },
}
```

- the rust compiler creates and manages the code and data structures for async code automatically.
- Ultimately, something has to execute a state machine, and that something is a **runtime**.

> Now you can see why the compiler stopped us from making main itself an async function back in Listing 17-3. If main were an async function, something else would need to manage the state machine for whatever future main returned, but main is the starting point for the program! Instead, we called the trpl::block_on function in main to set up a runtime and run the future returned by the async block until it’s done

---> Race both urls

```rust
use trpl::{self, Either, Html};

async fn page_title(url: &str) -> (&str, Option<String>) {
    let response_text = trpl::get(url).await.text().await;

    let title = Html::parse(&response_text)
        .select_first("title")
        .map(|title| title.inner_html());

    (url, title)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    trpl::block_on(async {
        //creates futures: not executed yet (futures are lazy)
        let title_fut_1 = page_title(&args[1]);
        let title_fut_2 = page_title(&args[2]);

        //futures are executed here cos of the select funtion
        //select returns Either enum
        let (url, maybe_title) = match trpl::select(title_fut_1, title_fut_2).await {
            Either::Left(left) => left,
            Either::Right(right) => right,
        };

        println!("{url} returned first");
        match maybe_title {
            Some(title) => println!("Its page title was: '{title}'"),
            None => println!("It had no title"),
        }
    })
}
```

```
async fn foo()
      ↓
  creates Future

foo().await
      ↓
  waits for/drives Future
  (inside an async context)

block_on(foo())
      ↓
  executor drives Future
  from synchronous code
```

## Applying Concurrency with Async

### Creating a New task with `spawn_task`

```rust
use std::time::Duration;

fn main() {
    trpl::block_on(async {
        let handle = trpl::spawn_task(async {
            for i in 1..10 {
                println!("hi no {i} from the first task");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        });

        for i in 1..5 {
            println!("hi no {i} from the second task");
            trpl::sleep(Duration::from_millis(500)).await;
        }

        //analogous to handle.join in thread
        //wait for the async block (upar vaala) to complete
        //else program terminates, when main ends
        //like after the below for loop ends and the upper async block
        //goes to sleep again the control is handed back to the main
        //and it goes out of scope. why does it not go out of scope before
        //cos block_on blocks the current thread until this future runs to
        //completion
        handle.await.unwrap();
    });
}
```

- we can do the above, without spawning a task: Because async blocks compile to anonymous futures, we can put each loop in an async block and have the runtime run them both to completion using the trpl::join function.

```rust
        let fut1 = async {
            for i in 1..10 {
                println!("hi number {i} from the first task!");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let fut2 = async {
            for i in 1..5 {
                println!("hi number {i} from the second task!");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        //produces a single future whose output is a tuple containing the output
        //of two futures
        trpl::join(fut1, fut2).await;
```

- `trpl::join` is fair, meaning it checks each future equally often, alternating between them and never lets one race ahead of them if the other is ready.
- With threads, the os decides which thread to run and for how long. With Async, runtime decides which task to check.

### Sending Data Between Two Tasks using Message Passing

```rust
        //trpl channel: Async version of mpsc channel
        //async version uses a mutable receiver instead
        let (tx, mut rx) = trpl::channel();

        //we dont have to spawn a separate task or even a separate thread
        //to pass the message; we just have to await the recv call
        let val = String::from("hi");
        tx.send(val).unwrap();

        //recv method produces a future so we need to await it
        let received = rx.recv().await.unwrap();
        println!("received '{received}'");
```

> The synchronous Receiver::recv method in std::mpsc::channel blocks until it receives a message. The trpl::Receiver::recv method does not, because it is async. Instead of blocking, it hands control back to the runtime until either a message is received or the send side of the channel closes. By contrast, we don’t await the send call, because it doesn’t block. It doesn’t need to, because the channel we’re sending it into is unbounded.

- In the above example the messges arrive right away + there's no concurrency yet

"All these codeblocks are wrapped in `block_on`"

```rust
        let (tx, mut rx) = trpl::channel();

        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("future"),
        ];

        for val in vals {
            tx.send(val).unwrap();
            trpl::sleep(Duration::from_millis(500)).await;
        }

        //no for loop for async so use while let
        //rx.recv produces a future and the runtime will pause
        //the future until it's ready. Once a message arrives, the
        //future will resolve to Some(message) until something arrives.
        while let Some(value) = rx.recv().await {
            println!("received '{value}'");
        }
```

- the above eg still has a couple problems;
  1. All 4 messages arrive together after 2k ms
  2. The program never exits

#### Codes within One Async Block Execute Linearly

- Within A given async block, the order in which `await` keywords appear in the code is also the order in which they're executed when the program runs.
- there's only one async block, so everything runs linearly

- to get the behavior we want, put tx and rx in their own async blocks so that the runtime can execute both of them separately

```rust
        let tx_fut = async {
            let vals = vec![
                String::from("hi"),
                String::from("from"),
                String::from("the"),
                String::from("future"),
            ];

            for val in vals {
                tx.send(val).unwrap();
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let rx_fut = async {
            while let Some(value) = rx.recv().await {
                println!("received '{value}'");
            }
        };

        trpl::join(tx_fut, rx_fut).await;

```

#### Moving Ownership into an Async Block

(trace the above prg and see why prg not terminating)
Right now, the async block where we send the messages only borrows tx because sending a message doesn’t require ownership, but if we could move tx into that async block, it would be dropped once that block ends.

-> So, just change the thingy to async move

```rust
        let (tx, mut rx) = trpl::channel();

        let tx_fut = async move {
            // --snip--

```

#### Joining a no. of futures with the `join!` Macro

This async channel is also a multiple-producer channel, so we can call clone on tx if we want to send messages from multiple futures, as shown in Listing 17-13.

```rust
        let (tx, mut rx) = trpl::channel();

        let tx1 = tx.clone();
        let tx1_fut = async move {
            let vals = vec![
                String::from("hi"),
                String::from("from"),
                String::from("the"),
                String::from("future"),
            ];

            for val in vals {
                tx1.send(val).unwrap();
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let rx_fut = async {
            while let Some(value) = rx.recv().await {
                println!("received '{value}'");
            }
        };

        let tx_fut = async move {
            let vals = vec![
                String::from("more"),
                String::from("messages"),
                String::from("for"),
                String::from("you"),
            ];

            for val in vals {
                tx.send(val).unwrap();
                trpl::sleep(Duration::from_millis(1500)).await;
            }
        };

        trpl::join!(tx1_fut, tx_fut, rx_fut);

```

## Working with any no. of Futures

Recall from the “Our First Async Program” section that at each await point, Rust gives a runtime a chance to pause the task and switch to another one if the future being awaited isn’t ready. The inverse is also true: Rust only pauses async blocks and hands control back to a runtime at an await point. Everything between await points is synchronous.

- `yield_now`, gives control to the runtime

```rust
        let a = async {
            println!("'a' started.");
            slow("a", 30);
            trpl::yield_now().await;
            slow("a", 10);
            trpl::yield_now().await;
            slow("a", 20);
            trpl::yield_now().await;
            println!("'a' finished.");
        };

        let b = async {
            println!("'b' started.");
            slow("b", 75);
            trpl::yield_now().await;
            slow("b", 10);
            trpl::yield_now().await;
            slow("b", 15);
            trpl::yield_now().await;
            slow("b", 350);
            trpl::yield_now().await;
            println!("'b' finished.");
        };

```

### Building Our Own Async Abstractions

We can also compose futures together to create new patterns. For example, we can build a timeout function with async building blocks we already have. When we’re done, the result will be another building block we could use to create still more async abstractions.

```rust
extern crate trpl; // required for mdbook test

use std::time::Duration;

fn main() {
    trpl::block_on(async {
        let slow = async {
            trpl::sleep(Duration::from_secs(5)).await;
            "Finally finished"
        };

        match timeout(slow, Duration::from_secs(2)).await {
            Ok(message) => println!("Succeeded with '{message}'"),
            Err(duration) => {
                println!("Failed after {} seconds", duration.as_secs())
            }
        }
    });
}
```

read book read book

## Streams: Futures in Sequence

Recall how we used the receiver for our async channel earlier in this chapter in the “Message Passing” section. The async recv method produces a sequence of items over time. This is an instance of a much more general pattern known as a stream. Many concepts are naturally represented as streams: items becoming available in a queue, chunks of data being pulled incrementally from the filesystem when the full data set is too large for the computer’s memory, or data arriving over the network over time. Because streams are futures, we can use them with any other kind of future and combine them in interesting ways. For example, we can batch up events to avoid triggering too many network calls, set timeouts on sequences of long-running operations, or throttle user interface events to avoid doing needless work.

- A stream is like an async form of an iteration

- We can create a stream from any iterator
- `stream` and `streamExt` trait

```rust
extern crate trpl; // required for mdbook test

use trpl::StreamExt;

fn main() {
    trpl::block_on(async {
        let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        // --snip--
        let iter = values.iter().map(|n| n * 2);
        let mut stream = trpl::stream_from_iter(iter);

        while let Some(value) = stream.next().await {
            println!("The value was: {value}");
        }
    });
}
```

## A Closer Look at the Traits for Async

### The Future Trait

Rust defines the future trait like this:

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

enum poll

```rust
pub enum Poll<T> {
    Ready(T), //indicates future is completed and T is available
    Pending, //indicates that the future still has work to do
}
```

> Note: It’s rare to need to call poll directly, but if you do need to, keep in mind that with most futures, the caller should not call poll again after the future has returned Ready. Many futures will panic if polled again after becoming ready. Futures that are safe to poll again will say so explicitly in their documentation. This is similar to how Iterator::next behaves.

- When you see code that uses await, Rust compiles it under the hood to code that calls poll.

- a runtime polls each future it is responsible for, putting the future back to sleep when it is not yet ready.

### The `pin` Type and the `Unpin` Trait

- It's common to have a collection such as a vec containing some no. of futures that wont be known until runtime.

```rust
        let tx_fut = async move {
            // --snip--
        };

        //put each future within a box to treat them as trait objects
        //this lets us treat anonymous futures produced by these types as
        //the same type, cos all of em implement the future trait
        let futures: Vec<Box<dyn Future<Output = ()>>> =
            vec![Box::new(tx1_fut), Box::new(rx_fut), Box::new(tx_fut)];

        trpl::join_all(futures).await;
```

output

```
error[E0277]: `dyn Future<Output = ()>` cannot be unpinned
  --> src/main.rs:48:33
   |
48 |         trpl::join_all(futures).await;
   |                                 ^^^^^ the trait `Unpin` is not implemented for `dyn Future<Output = ()>`
   |
   = note: consider using the `pin!` macro
           consider using `Box::pin` if you need to access the pinned value outside of the current scope
   = note: required for `Box<dyn Future<Output = ()>>` to implement `Future`
note: required by a bound in `futures_util::future::join_all::JoinAll`
  --> file:///home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/futures-util-0.3.30/src/future/join_all.rs:29:8
   |
27 | pub struct JoinAll<F>
   |            ------- required by a bound in this struct
28 | where
29 |     F: Future,
   |        ^^^^^^ required by this bound in `JoinAll`
```

> Directly awaiting a future with await pins the future implicitly. That’s why we don’t need to use pin! everywhere we want to await futures.

The note in the error message says that we need to use the pin! macro to _pin_ values. this guarantees that the pinned vals don't move in the memory (read error message, self explanatory)

The `trpl::join_all` function returns a struct called `JoinAll` . That struct is generic over a type F, which is constrained to implement the Future trait. Directly awaiting a future with await pins the future implicitly. That’s why we don’t need to use pin! everywhere we want to await futures.

However, we’re not directly awaiting a future here. Instead, we construct a new future, JoinAll, by passing a collection of futures to the join_all function. The signature for join_all requires that the types of the items in the collection all implement the Future trait, and Box<T> implements Future only if the T it wraps is a future that implements the Unpin trait.

-> Let's digin a bit deeper
look again at the definition of the future trait

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Future {
    type Output;

    // Required method
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

- the cx param and the context types are key to how the runtime actually knows when to check any given future while still being lazy. (deets out of scope)

-> Focus of `self` with type annotations now
A type annotation for self works like type annotations for other function parameters but with two key differences:

- It tells Rust what type self must be for the method to be called.
- It can’t be just any type. It’s restricted to the type on which the method is implemented, a reference or smart pointer to that type, or a Pin wrapping a reference to that type.

We’ll see more on this syntax in Chapter 18. For now, it’s enough to know that if we want to poll a future to check whether it is Pending or Ready(Output), we need a Pin-wrapped mutable reference to the type.

Pin is a wrapper for pointer-like types such as &, &mut, Box, and Rc. (Technically, Pin works with types that implement the Deref or DerefMut traits, but this is effectively equivalent to working only with references and smart pointers.) Pin is not a pointer itself and doesn’t have any behavior of its own like Rc and Arc do with reference counting; it’s purely a tool the compiler can use to enforce constraints on pointer usage.

Remember from earlier in this chapter that a series of await points in a future get compiled into a state machine, and the compiler makes sure that state machine follows all of Rust’s normal rules around safety, including borrowing and ownership. To make that work, Rust looks at what data is needed between one await point and either the next await point or the end of the async block. It then creates a corresponding variant in the compiled state machine. Each variant gets the access it needs to the data that will be used in that section of the source code, whether by taking ownership of that data or by getting a mutable or immutable reference to it.

- `Unpin` is a marker trait, similar to `Send` and `Sync` and thus has no functionality of its own. Marker traits only exist to tell the compiler that it's safe to use a type implementing the given trait in a particular context. `Unpin` tells the compiler that a given type does **NOT** need to uphold any guarantees about whether the value in question can be safely moved.

Just as with `Send` and `Sync`, the compiler implements `Unpin` automatically for all types where it can prove it is safe. A special case, again similar to `Send` and `Sync`, is where `Unpin` is _not_ implemented for a type. The notation for this is `impl !Unpin for SomeType`, where `SomeType` is the name of a type that does need to uphold those guarantees to be safe whenever a pointer to that type is used in a `Pin`.

> In other words, there are two things to keep in mind about the relationship between `Pin` and `Unpin`. First, `Unpin` is the “normal” case, and `!Unpin` is the special case. Second, whether a type implements `Unpin` or `!Unpin` only matters when you’re using a pinned pointer to that type like `Pin<&mut SomeType>`.

### The `Stream` Trait

Let’s review the definitions of the Iterator and Future traits before looking at how a Stream trait might merge them together. From Iterator, we have the idea of a sequence: its next method provides an Option<Self::Item>. From Future, we have the idea of readiness over time: its poll method provides a Poll<Self::Output>. To represent a sequence of items that become ready over time, we define a Stream trait that puts those features together:

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

trait Stream {
    type Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>>;
}
```

The `Stream` trait defines an associated type called `Item` for the type of the items produced by the stream. This is similar to `Iterator`, where there may be zero to many items, and unlike `Future`, where there is always a single `Output`, even if it’s the unit type `()`.

`Stream` also defines a method to get those items. We call it `poll_next`, to make it clear that it polls in the same way `Future::poll` does and produces a sequence of items in the same way `Iterator::next` does. Its return type combines `Poll` with `Option`. The outer type is `Poll`, because it has to be checked for readiness, just as a future does. The inner type is `Option`, because it needs to signal whether there are more messages, just as an iterator does.

`StreamExt` provides the `next` method.

> The StreamExt trait is also the home of all the interesting methods available to use with streams. StreamExt is automatically implemented for every type that implements Stream, but these traits are defined separately to enable the community to iterate on convenience APIs without affecting the foundational trait.
