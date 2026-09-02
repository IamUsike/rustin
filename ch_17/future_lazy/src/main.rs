/*
Prove that futures are lazy
- Write an async fn that prints "I am running!" and returns 42.
- Call it without awaiting. Just call the fn and store the res in a var.
- then await it. Explain why this would be impossible in js.
*/

async fn log() -> u32 {
    println!("Log running");
    42
}

#[tokio::main]
async fn main() {
    let a = log().await;
    println!("{a}");
}

/*
- in js the function would automatically run unlike rust cos rust lazy loads(calls?)
- when an async fn is called in rust, the fn body isnt executed. Instead, calling an
'async fn' returns a value repping the op
- to actually run the op we need to use the '.await' operator.
- in js, an async fn starts execution upon calling. await only waits the current block
to wait for that async op to finish

*/
