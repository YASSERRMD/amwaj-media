//! Prometheus metrics exporter

use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub async fn start_metrics_server(
    addr: SocketAddr,
    registry: prometheus::Registry,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Prometheus metrics server listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let registry = registry.clone();

        tokio::spawn(async move {
            let encoder = TextEncoder::new();
            let metric_families = registry.gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                buffer.len(),
                String::from_utf8_lossy(&buffer)
            );

            use tokio::io::AsyncWriteExt;
            let mut stream = stream;
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}
