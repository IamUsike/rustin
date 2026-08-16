## Shared counter, done right

Spawn 5 threads. Each increments a shared counter 1000 times. Print the final value — it must be exactly 5000. First try it with a plain i32 (watch the compiler reject it). Then with `Rc<RefCell<i32>>` (watch it reject that too). Then with `Arc<Mutex<i32>>` (this one works). Understand each rejection message before moving on.

> Cements: why Rc isn't Send, why RefCell isn't Sync, why Arc+Mutex is the right tool
