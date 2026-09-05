# More async/await topics

> runtimes usually provide a convenience attribute for tests similar to the one for
> `async main`.

## Blocking and cancellation

### Blocking IO

We say a thread (note we’re talking about OS threads here, not async tasks) is blocked when it can’t make any progress. That’s usually because it is waiting for the OS to complete a task on its behalf (usually I/O). Importantly, while a thread is blocked, the OS knows not to schedule it so that other threads can make progress. This is fine in a multithreaded program because it lets other threads make progress while the blocked thread is waiting. However, in an async program, there are other tasks which should be scheduled on the same OS thread, but the OS doesn’t know about those and keeps the whole thread waiting. This means that rather than the single task waiting for its I/O to complete (which is fine), many tasks have to wait (which is not fine).

### Blocking Computation

- if there's a long computation, the task wont yield control to the runtime and continues execution. (blocking the thread)

#### cancellation

- means stopping a future from executing.

> Since in Rust (and in contrast to many other async/await systems), futures must be driven forward by an external force (like the async runtime), if a future is no longer driven forward then it will not execute any more. If a future is dropped (remember, a future is just a plain old Rust object), then it can never make any more progress and is canceled.

cancellation can be initiated in many ways:

- by simply dropping a future
- calling `abort` on a task's `JoinHandle`(or an `Aborthandle`) | (tokio specific)
- Via a CancellationToken (which requires the future being canceled to notice the token and cooperatively cancel itself). | (tokio specific)
- implicitly by a fn or macro like `select`

except for the case with `cancellationToken`. The futures wont receive any notice to clean up(besides its destructor).

From the perspective of writing async code (in async functions, blocks, futures, etc.), the code might stop executing at any await (including hidden ones in macros) and never start again. In order for your code to be correct (specifically to be cancellation safe), it must work correctly whether it completes normally or whether it terminates at any await point1.

```rust
async fn some_function(input: Option<Input>) {
    let Some(input) = input else {
        return;           // Might terminate here (`return`).
    };

    let x = foo(input)?;  // Might terminate here (`?`).

    let y = bar(x).await; // Might terminate here (`await`).

    // ...

    //                       Might terminate here (implicit return).
}
```

### Async blocks

Unfortunately, control flow with async blocks is a little quirky. Because an async block creates a future rather than straightforwardly executing, it behaves more like a function than a regular block with respect to control flow. `break` and `continue` cannot go ‘through’ an async block like they can with regular blocks; instead you have to use return:

```rust
loop {
    {
        if ... {
            // ok
            continue;
        }
    }

    async {
        if ... {
            // not ok
            // continue;

            // ok - continues with the next execution of the `loop`, though note that if there was
            // code in the loop after the async block that would be executed.
            return;
        }
    }.await
}
```

To implement break you would need to test the value of the block (a common idiom is to use ControlFlow for the value of the block, which also allows use of ?).

-> [async closures](https://www.reddit.com/r/rust/comments/1hd3ivt/async_closures_stabilized/)
