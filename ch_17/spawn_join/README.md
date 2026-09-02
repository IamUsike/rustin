# tokio::spawn vs tokio::join!

Spawn 3 tasks with tokio::spawn — each sleeps a different amount and prints when done. Collect the JoinHandles and await them. Then do the same thing with join!. Write a comment explaining the key difference: spawn creates an independent task that runs even if you don't await the handle. join! runs futures concurrently but YOU must drive them. Drop a JoinHandle without awaiting it — what happens?
