struct Db_conn {
    url: String,
    password: String,
}

use std::rc::Rc;

//let me create 3 db connections at a time (assume multithreaded some gawar shit)
fn main() {
    let url = String::from("urlman");
    let password = String::from("pwman");

    let conn = Rc::new(Db_conn { url, password });
    let c1 = Rc::clone(&conn);
    println!("after c1: {}", Rc::strong_count(&conn));

    let c2 = Rc::clone(&conn);
    println!("after c2: {}", Rc::strong_count(&conn));

    let c3 = Rc::clone(&conn);
    println!("after c3: {}", Rc::strong_count(&conn));
}

// We create ONE DbConn on the heap and wrap it in an Rc (Reference Counted pointer).
//
// Rc allows multiple parts of the program to own the same value without copying it.
// Calling Rc::clone() DOES NOT clone the DbConn itself—it only creates another Rc
// pointing to the same allocation and increments the reference count.
//
// Heap:
//
//              +-----------------------------+
//              | DbConn                      |
//              | url: "urlman"               |
//              | password: "pwman"           |
//              +-----------------------------+
//                       ▲
//            ┌──────────┼──────────┐
//            │          │          │
//          conn        c1         c2         c3
//
// Strong count:
// After creation : 1 (conn)
// After c1       : 2
// After c2       : 3
// After c3       : 4
//
// The DbConn is automatically deallocated only when the last Rc owner is dropped.
//
// NOTE:
// Rc is for SINGLE-THREADED shared ownership.
// For MULTI-THREADED shared ownership, use Arc<T> instead.
