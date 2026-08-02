** hard **

### Parallel word count

Count total words across 5 hardcoded strings. Spawn one thread per string, have each thread count words and send the result back via an mpsc channel. Sum results in the main thread.
