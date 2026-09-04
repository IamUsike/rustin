# Lifetimes across await -- why references don't work

Write an async fn that takes a `&str`, does some work (sleep), then uses the `&str` again after the await. watch it compile fine. Now try to spawn that future with `tokio::spawn` -- watch the compiler reject it with "static required".

Understand WHY: spawned tasks can outlive caller's stack frame, so they cant borrow local data.

Fix it three ways:

1. clone the string
2. use `Arc<str>`
3. use `'static` data
