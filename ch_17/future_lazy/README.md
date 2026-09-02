## Prove that futures are lazy

Write an async fn that prints "I am running!" and returns 42. Call it WITHOUT awaiting — just call the function, store the result in a variable. Notice nothing prints. Then await it. Notice it prints. Add a comment explaining why this would be impossible in JS (a JS async fn always starts immediately when called).

---

```rust
/*
Prove that futures are lazy
- Write an async fn that prints "I am running!" and returns 42.
- Call it without awaiting. Just call the fn and store the result in a variable.
  Notice that nothing prints.
- Then await it. Notice that it prints.
- Explain why this would be impossible in JS.
*/

async fn log() -> u32 {
    println!("I am running!");
    42
}

#[tokio::main]
async fn main() {
    // Calling an async fn in Rust does NOT execute its body.
    // It returns a Future representing the operation.
    //
    // Unlike the original:
    //     let a = log().await;
    //
    // where we immediately awaited the Future, here we first
    // store the Future without awaiting it.
    let future = log();

    println!("Before await");

    // The Future is now awaited, so the executor starts polling it
    // and the body of `log()` executes.
    let a = future.await;

    println!("{a}");
}

/*
Contrast with JavaScript:

In Rust:
    let future = log();

    // Nothing happens yet.
    // `log()` returns a Future and its body has not started executing.

    let result = future.await;
    // The Future is driven toward completion here.

In JavaScript:
    const promise = log();

    // `log()` starts executing immediately when called.
    // "I am running!" would already have been printed here.
    //
    // `await` only waits for the Promise to settle; it does not
    // cause the async function to start running.

So:

Rust async fn call  -> creates a lazy Future
Rust .await         -> drives the Future toward completion

JS async fn call    -> starts executing immediately
JS await            -> waits for the Promise
*/
```
