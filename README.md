# pingpong

```
cargo run --release -- --listen-port 12345 --send-port 23456 --send-address 127.0.0.1 --data-length 16 --send-first
```

```
cargo run --release -- --listen-port 23456 --send-port 12345 --send-address 127.0.0.1 --data-length 16
```