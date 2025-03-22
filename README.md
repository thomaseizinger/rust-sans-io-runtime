# sairun - **sa**ns-**I**O async **run**time

sairun is an experimental async runtime for sans-IO implementations.
sans-IO is cool but it is annoying that we have to write our own state machines for handling what are essentially async operations.
It would be much better if we could just use async/await and let the compiler handle composing for us.

With sairun, you can now do this!

Essentially, `sairun` is just a sans-IO component itself that acts as a bridge between its `Future`s and your actual IO code.

```rust
async fn greet() {
    let src = "127.0.0.1:1234".parse().unwrap();
    let dst = "192.168.0.1:5678".parse().unwrap();

    sairun::udp::send_to(src, dst, "Hello".as_bytes().to_vec()).await;

    let msg = sairun::udp::recv_from(src, dst).await;
    let msg = String::from_utf8(msg).unwrap();

    println!("Received '{msg}' from {dst}");
}

#[test]
fn send_receive() {
    let mut runtime = sairun::Runtime::default();

    runtime.spawn(greet(), Instant::now());

    loop {
        if let Some(msg) = runtime.poll_datagram() {
            // Use an actual socket to send the UDP datagram
        }

        let (local, remote, msg) = todo!("receive datagram from a socket");

        runtime.handle_input(local, remote, msg, Instant::now());
    }
}
```
