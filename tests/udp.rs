use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Instant,
};

use sairun::{Runtime, udp};

const SRC1: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 1234));

const DST1: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 1), 5678));
const DST2: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 2), 5678));

#[test]
fn smoke() {
    let now = Instant::now();
    let mut runtime = Runtime::default();

    runtime.spawn(
        async move {
            udp::send_to(SRC1, DST1, "Hello".as_bytes().to_vec()).await;

            let buf = udp::recv_from(SRC1, DST1).await;

            let msg = String::from_utf8(buf).unwrap();

            println!("Received '{msg}' from {DST1}")
        },
        now,
    );
    runtime.spawn(
        async move {
            udp::send_to(SRC1, DST2, "Hello".as_bytes().to_vec()).await;

            let buf = udp::recv_from(SRC1, DST2).await;

            let msg = String::from_utf8(buf).unwrap();

            println!("Received '{msg}' from {DST2}")
        },
        now,
    );

    runtime.handle_input(SRC1, DST1, "world".as_bytes().to_vec(), now);
    runtime.handle_input(SRC1, DST2, "foo".as_bytes().to_vec(), now);

    while !runtime.is_finished() {
        runtime.tick(now);
    }
}
