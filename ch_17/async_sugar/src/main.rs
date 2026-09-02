/*
Write the same fn in 2 ways
A. Write an async fn that sleeps for 1sec and returns a string
B. A regular fn that returns impl Future<Output = String> using async move {} block
*/

use tokio::time::{Duration, sleep};

async fn aa() -> String {
    sleep(Duration::from_millis(5000)).await;
    println!("aa slept for 5 secd");
    String::from("aa return")
}

fn bb() -> impl Future<Output = String> {
    async move {
        sleep(Duration::from_millis(2000)).await;
        println!("bb slept for 2 secs");
        String::from("bb return")
    }
}

#[tokio::main]
async fn main() {
    //main actually stops here and awaits for that thread to finish
    aa().await;
    println!("main after a");

    bb().await;
    println!("main after bb");
}

/*
- as desugared into something like how b is implemented
- the indepth explanation of how that works can be found here https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html
- The state machine that's used to track would be an enum similar to

enum PageTitleFuture<'a> {
    Initial { url: &'a str },
    GetAwaitPoint { url: &'a str },
    TextAwaitPoint { response: trpl::Response },
}

*/
