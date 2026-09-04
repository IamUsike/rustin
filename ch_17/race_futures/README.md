# select! — race futures against each other

Write two async tasks: one that completes after 200ms ("fast result"), one after 2000ms ("slow result"). Use tokio::select! to take whichever completes first and cancel the other. Then build a practical version: a "fetch with timeout" pattern — select! between your async work and a tokio::time::sleep(timeout). If sleep wins, return an Err("timed out").
