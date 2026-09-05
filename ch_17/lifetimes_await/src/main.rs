use tokio::time::{Duration, sleep};

async fn _soja(s: &str) {
    println!("async fn going to sleep with {s}");
    sleep(Duration::from_secs(3)).await;
    println!("async fn finished sleeping with {s}");
}

#[tokio::main]
async fn main() {
    let s = "Don't sleep with me";
    // soja(s).await; //works fine
    // tokio::spawn(async { soja(s).await }); //why does this not throw an error?

    // tokio::spawn(async {
    //     println!("task going to sleep with {s}");
    //     sleep(Duration::from_secs(3)).await;
    //     println!("async fn finished sleeping with {s}");
    // });

    //the above spawn throws an error:      ├╴  async block may outlive the current function, but it borrows `s`, which is owned by the current function
    //I'm not sure why? I went through docs and shi but couldn't get it
    //but based on the error message, i presume the async fn gets cleaned
    //up when the callee goes out of scope but in case of tokio::spawn
    //the task stays alive until that task itself terminates or  we manually
    //terminate it (so the error might outlive the current function)

    // sol 1: Clone the string
    // this is better in case the data structure is small +
    // when we dont necessarily have any modifications to make
    // on that data. V simple
    let task = tokio::spawn(async move {
        println!("task going to sleep with {s}");
        sleep(Duration::from_secs(3)).await;
        println!("async fn finished sleeping with {s}");
    });

    let _ = tokio::join!(task);
    println!("{s}"); //we moved s into task but how we able to print it here? 

    //I get how to use arc and 'static
    //we can use arc when multiple tasks are there and they all want to edit the same
    //data. (along with a mutex obv )
    //with static, the data will live for as long as the program exists so that's there
}
