use axum::{ routing::get, Router};
use hyper::server::{
    accept::Accept,
    conn::{AddrIncoming, Http},
};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};
use tokio::net::{TcpListener, UdpSocket};

use tower::make::MakeService;
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

// fn init(listener: TcpListener, protocol: Arc<Http>) {
//     // let protocol = Arc::new(Http::new());
//     loop {
//         let stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx))
//             .await
//             .unwrap()
//             .unwrap();

//         let acceptor = acceptor.clone();
//         let protocol = protocol.clone();

//         let svc = MakeService::<_, Request<hyper::Body>>::make_service(&mut app, &stream);

//         tokio::spawn(async move {
//             if let Ok(stream) = acceptor.accept(stream).await {
//                 let _ = protocol.serve_connection(stream, svc.await.unwrap()).await;
//             }
//         });
//     }
// }
