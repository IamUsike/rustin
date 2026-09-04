# Concurrent Programming

## processes and threads

- There is one **process** per executable, so supporting multiple processes means a program can run multiple programs concurrently.
- there can be multiple threads per process => concurrency can exist within a process.

- memory is shared between threads but not processes => communication happens bw processes by some kind of message passing.
- from a programs perspective, a the single process is their whole world.

- The OS is responsible for scheduling threads.

> Pre-emptive multi-tasking, Interleaving
> Or more verbosely, the OS pre-emptively pauses execution. It is pre-emptive because the OS is pausing the thread to make time for another thread, before the first thread would otherwise pause, to ensure that the second thread can execute before it becomes a problem that it can’t

- when an OS pauses one thread and starts another(for any reason), it is called _context switching_.

## Async Programming

similar to concurrency between threads by os. 2 main diffs

1. The concurrency is completely managed by the program with no help from the OS
2. The multitasking is co-operative rather than pre-emptive.

_To distinguish from threads we'll call the seq of execs in async concurrency as tasks_

- With threads, the thread doing IO requests IO from the OS, the thread is paused by the OS, other threads get work done, and when the IO is done, the OS wakes up the thread so it can continue execution with the result of the IO.
- With async, the task doing IO requests IO from the runtime, the runtime requests IO from the OS but the OS returns control to the runtime. The runtime pauses the IO task and schedules other tasks to get work done. When the IO is done, the runtime wakes up the IO task so it can continue execution with the result of the IO.

> the advantage of using async is that overheads are much lower (no constant switching between threads), that systems can suppport orders of magnitude more tasks than threads.

## Concurrency and Parallelism

Concurrency is about ordering of computations and parallelism is about the mode of execution.
