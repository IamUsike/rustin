# Arc + Mutex in async (and why Mutex::lock can deadlock)

Build a shared async counter: Arc<Mutex<u32>>. Spawn 5 tasks each incrementing 1000 times. BUT: deliberately cause a deadlock by holding the std::sync::MutexGuard across an .await point (lock, then sleep while locked). Observe the hang. Then fix it by either: (1) using tokio::sync::Mutex instead, (2) dropping the guard before .await. Explain why holding std Mutex across .await is dangerous in async code.
