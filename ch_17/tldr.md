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
