/* Concurrent file downloader
- Simulate downloading 6 urls concurrently
- each download is an async fn that takes a "url: &str" and "size_kb: u32",
  sleeps `size_kb*10` ms and returns a string result.
- use tokio::spawn for all 6, collect results, await all.
- print results as they complete using JoinSet, so results are processed
  as soon as they are ready, not in the submission order
*/

use tokio;
use tokio::task::JoinSet;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let urls = vec![
        ("one", 120),
        ("two", 40),
        ("three", 280),
        ("four", 200),
        ("five", 160),
        ("six", 80),
    ];
    // let mut tasks = Vec::new();
    let mut tasks = JoinSet::new();
    //Is it possible to send JoinHandle(returned by tokio::spawn) to JoinSet and
    //then print whichever finishes first, so using JoinSet to spawn directly
    for (url, size_kb) in urls {
        let task = tasks.spawn(async move { download(url, size_kb).await });
    }

    //wait for whatever task finishes and polls
    while let Some(Ok(res)) = tasks.join_next().await {
        // let output = res.unwrap();
        println!("{:?}", output);
        println!("task op received");
    }
}

//check Option<Option<&str>> later
async fn download(url: &str, size_kb: u64) -> String {
    let ret_str = String::from(url) + &String::from("Downloaded");
    println!("Downloading {url}");
    sleep(Duration::from_millis(size_kb * 10)).await;
    return ret_str;
}
