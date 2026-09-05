# Rust async concepts

## The runtime

A runtime must interact with OS to manage async IO and it must also provide functionality for timer management.

- _reactor/event loop/driver_(equivalent terms) dispatches IO and timer events, interacts with the OS, and does the lowest-level driving forward of execution.
- _scheduler_: determines which tasks execute and on which OS threads.
- _executor/runtime_: combines reactor and scheduler and is the userfacing API for running async tasks; runtime is also used to mean the whole lib or functionality.

### futures and tasks

- the basic unit of async concurrency in rust is the _future_.
- it's just an object(struct or enum) that implements the `Future` trait.

```
I’ve used the term ‘async task’ quite a bit in an informal way in the previous chapter and this one. I’ve used the term to mean a logical sequence of execution; analogous to a thread but managed within a program rather than externally by the OS. It is often useful to think in terms of tasks, however, Rust itself has no concept of a task and the term is used to mean different things! It is confusing! To make it worse, runtimes do have a concept of a task and different runtimes have slightly different concepts of tasks.

From here on in, I’m going to try to be precise about the terminology around tasks. When I use just ‘task’ I mean the abstract concept of a sequence of computation that may occur concurrently with other tasks. I’ll use ‘async task’ to mean exactly the same thing, but in contrast to a task which is implemented as an OS thread. I’ll use ‘runtime’s task’ to mean whatever kind of task a runtime imagines, and ‘tokio task’ (or some other specific runtime) to mean Tokio’s idea of a task.

An async task in Rust is just a future (usually a ‘big’ future made by combining many others). In other words, a task is a future which is executed. However, there are times when a future is ‘executed’ without being a runtime’s task. This kind of a future is intuitively a task but not a runtime’s task. I’ll spell this out more when we get to an example of it.
```

............... ok bro :smile

## Async functions

- the async fn's caller can choose not to wait for the fn to complete before doing something else.

### `.await`

We stated above that a future is a computation that will be ready at some point in the future. To get the result of that computation, we use the await keyword. If the result is ready immediately or can be computed without waiting, then await simply does that computation to produce the result. However, if the result is not ready, then await hands control over to the scheduler so that another task can proceed (this is cooperative multitasking mentioned in the previous chapter).

- futures are lazy, so they dont start exectuion until polled (await-ed)
- An important intuition about futures in Rust is that they are inert objects. To get any work done they must be driven forward by an external force (usually an async runtime).

> Just calling and awaiting async functions does not introduce any concurrency unless there are other tasks to schedule while the awaiting task is waiting.

## Spawning Tasks

```rust
use tokio::{spawn, time::{sleep, Duration}};

async fn say_hello() {
    // Wait for a while before printing to make it a more interesting race.
    sleep(Duration::from_millis(100)).await;
    println!("hello");
}

async fn say_world() {
    sleep(Duration::from_millis(100)).await;
    println!("world!");
}

#[tokio::main]
async fn main() {
    spawn(say_hello());
    spawn(say_world());
    // Wait for a while to give the tasks time to run.
    sleep(Duration::from_millis(1000)).await;
}
```

There are three concepts in play: futures, tasks, and threads. The spawn function takes a future (which remember can be made up of many smaller futures) and runs it as a new Tokio task. Tasks are the concept which the Tokio runtime schedules and manages (not individual futures). Tokio (in its default configuration) is a multi-threaded runtime which means that when we spawn a new task, that task may be run on a different OS thread from the task it was spawned from (it may be run on the same thread, or it may start on one thread and then be moved to another later on).

So, when a future is spawned as a task it runs concurrently with the task it was spawned from and any other tasks. It may also run in parallel to those tasks if it is scheduled on a different thread.

To summarise, when we write two statements following each other in Rust, they are executed sequentially (whether in async code or not). When we write await, that does not change the concurrency of sequential statements. E.g., foo(); bar(); is strictly sequential - foo is called and afterwards, bar is called. That is true whether foo and bar are async functions or not. foo().await; bar().await; is also strictly sequential, foo is fully evaluated and then bar is fully evaluated. In both cases another thread might be interleaved with the sequential execution and in the second case, another async task might be interleaved at the await points, but the two statements are executed sequentially with respect to each other in both cases.

If we use either thread::spawn or tokio::spawn we introduce concurrency and potentially parallelism, in the first case between threads and in the second between tasks.

### Joining tasks

`tokio::spawn`, returns `JoinHandle`, and we can await them (trpl explanations)

- `spawn` is not an async fn. It's just a regular fn that returns a future.
