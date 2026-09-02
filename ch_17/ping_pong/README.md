# Async Ping Pong

Recreate the Ch16 ping-pong exercise but with tokio::sync::mpsc. Two async tasks: task A sends "ping", task B receives it and sends "pong" back, task A receives "pong" and prints it. Notice how .await on channel send/recv lets other tasks run while waiting — unlike thread blocking. Add 5 rounds of ping-pong.
