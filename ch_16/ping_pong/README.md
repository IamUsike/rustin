## Ping pong

Create two channels: main→thread and thread→main. Main sends "ping", thread receives it, prints "got ping", sends back "pong", main receives and prints "got pong". That's it. Simple — but it forces you to set up two channels and understand that each channel is one-directional.

Cements: channels are one-way, you need two for two-way communication
