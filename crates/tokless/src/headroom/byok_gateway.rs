use crate::util::byok::byok_gateway_port;
use crate::util::logger::L;

pub const BYOK_ROUTE_HEADER: &str = "X-Tokless-Route";

pub async fn run_byok_gateway_serve() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = byok_gateway_port();
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    L.info(&format!("BYOK gateway listening on {}", addr));

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            // Echo / HTTP proxy handling
            let _ = stream;
        });
    }
}

