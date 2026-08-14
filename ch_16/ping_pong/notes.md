once again

## Transfer Data Between Threads with Message Passing

-> A popular approach to ensure safe concurrency
**Do not communicate by sharing memory; instead, share memory by communicating**

-> Channels are used to send data from one thread to another.
-> typically used to accomplish message sending concurrency.

-> A channel has 2 halves: transmitter & receiver.
-> one part of the code calls transmitter with the data you want to send while
the other part calls the receiver to receive that data.
-> A channel is said to be _closed_ if either the receiver or transmitter half is dropped.
