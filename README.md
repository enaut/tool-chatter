# Chatter

## a tool to collect all the chats (private and public) as an admin of BBB.

This tool is used in a pipe:

```rust
#!/bin/bash

grep "chatId" /var/log/bbb-apps-akka/* | grep "\"message\":\".*\"" | chatter > /chats
```