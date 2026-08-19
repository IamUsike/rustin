# Mini thread pool from scratch

Build a ThreadPool struct with new(n: usize) that spawns n worker threads, and execute(f: impl FnOnce() + Send + 'static) that sends a job to an available worker. Workers sit in a loop waiting for jobs via a shared channel. Use Arc<Mutex<Receiver>> so all workers share the same job queue (this is the one valid case for Mutex-wrapped Receiver). Implement Drop for ThreadPool that sends a shutdown signal and joins all threads. Test with 10 jobs on 3 workers.

> Cements: everything — spawn, move, Arc, Mutex, channels, FnOnce as a trait object, graceful shutdown
