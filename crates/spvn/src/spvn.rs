use std::sync::Arc;
use tokio_rustls::rustls::ServerConfig;

struct Spvn {
    tls: Option<Arc<ServerConfig>>,
}

impl Spvn {
    fn new() -> Arc<Self> {
        let app = Arc::new(Spvn { tls: None });
        app
    }
    fn start_service(self) {}
}
