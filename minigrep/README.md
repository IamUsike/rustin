> Rust’s speed, safety, single binary output, and cross-platform support make it an ideal language for creating command line tools, so for our project, we’ll make our own version of the classic command line search tool grep (globally search a regular expression and print). In the simplest use case, grep searches a specified file for a specified string. To do so, grep takes as its arguments a file path and a string. Then, it reads the file, finds lines in that file that contain the string argument, and prints those lines.

---

> For now, you only need to know two details about iterators: Iterators produce a series of values, and we can call the collect method on an iterator to turn it into a collection, such as a vector, which contains all the elements the iterator produces.
