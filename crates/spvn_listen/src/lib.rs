use tokio::net::{TcpListener, UdpSocket};

pub async fn spawn_tcp(addr: &str) -> TcpListener {
    let listener = TcpListener::bind(addr).await;

    match listener {
        Err(listener) => panic!("{}", listener),
        Ok(listener) => return listener,
    }
}

pub async fn spawn_socket(addr: &str) -> UdpSocket {
    let listener = UdpSocket::bind(addr).await;

    match listener {
        Err(listener) => panic!("{}", listener),
        Ok(listener) => return listener,
    }
}
