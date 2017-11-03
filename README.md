Laza Upatising

Read LISTEN:PORT until it sees an empty line ('\r\n'), then responds with "HTTP/1.1 200 OK"
```
$ cargo build --release
$ RUST_LOG=debug ./target/release/http_server LISTEN:PORT
```
