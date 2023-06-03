use std::net::SocketAddr;

use tokio::net::TcpListener;

pub async fn spawn_so_reuse(addr: SocketAddr) -> TcpListener {
    let sock = socket2::Socket::new(
        match addr {
            SocketAddr::V4(_) => socket2::Domain::IPV4,
            SocketAddr::V6(_) => socket2::Domain::IPV6,
        },
        socket2::Type::STREAM,
        None,
    )
    .expect("Failed to initialize socket");

    // SO_REUSE_PORT
    sock.set_reuse_port(true).unwrap();
    // SO_REUSE_ADDR
    sock.set_reuse_address(true).unwrap();
    sock.set_nonblocking(true).unwrap();
    sock.bind(&addr.into()).expect("Failed to bind to socket");
    sock.listen(addr.port().into())
        .expect("Failed to initialize listener");
    TcpListener::from_std(sock.into()).unwrap()
}
