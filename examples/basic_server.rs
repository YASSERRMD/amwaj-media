//! Basic server example

use amwaj_media::{config::Config, grpc::server::GrpcServer, metrics::Metrics};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create default config
    let config = Config::default();
    
    // Initialize metrics
    let metrics = Arc::new(Metrics::new(&config));
    
    // Create and start server
    let server = GrpcServer::new(config, metrics);
    
    tracing::info!("Starting basic server example...");
    server.start().await?;
    
    Ok(())
}
