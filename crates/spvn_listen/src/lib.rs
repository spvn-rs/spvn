use std::error::Error;
use tokio::net::{TcpListener};
use tokio::net::UdpSocket as AsyncUdpSocket;

pub async fn spawn_tcp(addr: &str) -> TcpListener {
    let listener = TcpListener::bind(addr).await;

    match listener {
        Err(listener) => panic!("{}", listener),
        Ok(listener) => return listener,
    }
}

pub async fn spawn_socket(addr: &str) -> AsyncUdpSocket {
    let listener = AsyncUdpSocket::bind(addr).await;

    match listener {
        Err(listener) => panic!("{}", listener),
        Ok(listener) => return listener,
    }
}
