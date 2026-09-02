/* SEQ VS CONCURRENT
- write an async fn simulate_fetch(id: u32, ms: u64) that sleeps for ms milliseconds and
  prints "fetch {id} done".
- ver A: await 3 fetches sequentially (total time = sum of all delays)
- ver B: run all 3 concurrently with tokio::join!

- time both with std::time::Instant. The diff in elapsed time is the point of async
*/

use std::time::Instant;
use tokio;
use tokio::time::{Duration, sleep};

async fn simulate_fetch(id: u32, ms: u64) {
    sleep(Duration::from_millis(ms)).await;
    println!("fetch {id} done");
}

#[tokio::main]
async fn main() {
    // Ver A: Await 3 fetches sequentially
    let now = Instant::now();

    simulate_fetch(1, 1000).await;
    simulate_fetch(2, 1000).await;
    simulate_fetch(3, 1000).await;

    let elapsed_time = now.elapsed();

    println!("Seq awaits took {}s to run", elapsed_time.as_secs());

    // Ver B: tokio::join!
    let now = Instant::now();

    tokio::join!(
        simulate_fetch(1, 1000),
        simulate_fetch(2, 1000),
        simulate_fetch(3, 1000),
    );

    let elapsed_time = now.elapsed();

    println!("Concurrent awaits took {}s to run", elapsed_time.as_secs());
}

/* OUTPUT
fetch 1 done
fetch 2 done
fetch 3 done
Seq awaits took 3s to run

fetch 2 done
fetch 3 done
fetch 1 done
Concurrent awaits took 1s to run

Just read the tokio::join!() docs to understand what's happening here
plus code is linear/sequential with the same async block
*/
