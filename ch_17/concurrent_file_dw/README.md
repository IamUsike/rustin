# Concurrent file "downloader"

Simulate downloading 6 URLs concurrently. Each "download" is an async fn that takes a url: &str and a size_kb: u32, sleeps size_kb * 10ms (simulating transfer time), and returns a String result. Use tokio::spawn for all 6, collect handles, await all. Print results as they complete using JoinSet (tokio::task::JoinSet) so you process each result as soon as it's ready, not in submission order.
