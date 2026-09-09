# Async streams — process data as it arrives

Use `tokio_stream` to create a stream of integers (`stream::iter`), then use `StreamExt` methods to: filter evens, map to squares, take first 5, collect. Then simulate a real stream: build one with `tokio::sync::mpsc` where a producer sends values every 100ms and the consumer processes them as a stream using `ReceiverStream.` This is `async` iteration — the `async` equivalent of iteratorse
