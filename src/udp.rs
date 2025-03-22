use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

/// Send a datagram to a remote address from a given socket.
pub fn send_to(src: SocketAddr, dst: SocketAddr, bytes: Vec<u8>) -> SendFuture {
    SendFuture { src, dst, bytes }
}

/// Receive a datagram on a given socket from a remote address.
pub fn recv_from(src: SocketAddr, dst: SocketAddr) -> RecvFromFuture {
    RecvFromFuture { src, dst }
}

/// Receive a datagram on a given socket.
///
/// Returns the remote address of the datagram along with the data.
pub fn recv(src: SocketAddr) -> RecvFuture {
    RecvFuture { src }
}

pub struct SendFuture {
    src: SocketAddr,
    dst: SocketAddr,
    bytes: Vec<u8>,
}

impl Future for SendFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        crate::runtime::ExtData::from_context(cx).buffer_udp_transmit(
            self.src,
            self.dst,
            std::mem::take(&mut self.bytes),
        );

        Poll::Ready(())
    }
}

pub struct RecvFuture {
    src: SocketAddr,
}

impl Future for RecvFuture {
    type Output = (SocketAddr, Vec<u8>);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match crate::runtime::ExtData::from_context(cx).take_datagram_by_src(self.src) {
            Some(datagram) => Poll::Ready(datagram),
            None => Poll::Pending,
        }
    }
}

pub struct RecvFromFuture {
    src: SocketAddr,
    dst: SocketAddr,
}

impl Future for RecvFromFuture {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match crate::runtime::ExtData::from_context(cx)
            .take_datagram_by_src_and_dst(self.src, self.dst)
        {
            Some(datagram) => Poll::Ready(datagram),
            None => Poll::Pending,
        }
    }
}
