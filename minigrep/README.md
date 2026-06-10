> Rust’s speed, safety, single binary output, and cross-platform support make it an ideal language for creating command line tools, so for our project, we’ll make our own version of the classic command line search tool grep (globally search a regular expression and print). In the simplest use case, grep searches a specified file for a specified string. To do so, grep takes as its arguments a file path and a string. Then, it reads the file, finds lines in that file that contain the string argument, and prints those lines.

---

> For now, you only need to know two details about iterators: Iterators produce a series of values, and we can call the collect method on an iterator to turn it into a collection, such as a vector, which contains all the elements the iterator produces.

### Separating Concerns in Binary Projects

The organizational problem of allocating responsibility for multiple tasks to the `main` function is common to many binary projects. As a result, many Rust programmers find it useful to split up the separate concerns of a binary program when the `main` function starts getting large. This process has the following steps:

- Split your program into a `main.rs` file and a `lib.rs` file and move your program’s logic to `lib.rs`.
- As long as your command line parsing logic is small, it can remain in the `main` function.
- When the command line parsing logic starts getting complicated, extract it from the `main` function into other functions or types.

The responsibilities that remain in the `main` function after this process should be limited to the following:

- Calling the command line parsing logic with the argument values
- Setting up any other configuration
- Calling a `run` function in `lib.rs`
- Handling the error if `run` returns an error

This pattern is about separating concerns: `main.rs` handles running the program and `lib.rs` handles all the logic of the task at hand. Because you can’t test the `main` function directly, this structure lets you test all of your program’s logic by moving it out of the `main` function. The code that remains in the `main` function will be small enough to verify its correctness by reading it. Let’s rework our program by following this process.
