## Spawn and join 5 threads

Spawn 5 threads. Each thread prints its own ID (0–4) and the square of that ID. In main, join all of them. Run it several times and observe that the print order changes — that's the point. Then modify it so threads sleep for (5 - id) * 100ms before printing, so thread 0 sleeps longest. What order do you expect now?

> Cements: thread::spawn, move closure, JoinHandle::join, nondeterminism

thread::spawnmove closureJoinHandle
