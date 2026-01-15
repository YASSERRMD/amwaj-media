//! gRPC server implementation

use crate::config::Config;
use crate::metrics::Metrics;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct GrpcServer {
    config: Config,
    #[allow(dead_code)]
    metrics: Arc<Metrics>,
}

impl GrpcServer {
    pub fn new(config: Config, metrics: Arc<Metrics>) -> Self {
        Self { config, metrics }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);

        tracing::info!("gRPC server starting on {}", addr);

        // Stub: Just bind to the port for now
        // Full gRPC service implementation will be added in Phase 2
        // after proto compilation is set up
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("Server listening on {}", addr);
        
        loop {
            let (socket, peer_addr) = listener.accept().await?;
            tracing::debug!("New connection from {}", peer_addr);
            // For now, just accept and drop connections
            // Real gRPC handling will be added in Phase 2
            drop(socket);
        }
    }
}
