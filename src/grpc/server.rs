//! gRPC server implementation

use crate::config::Config;
use crate::metrics::Metrics;
use crate::grpc::service::AmwajMediaService;
use std::sync::Arc;
use tokio::net::TcpListener;

/// gRPC Server for Amwaj Media
pub struct GrpcServer {
    config: Config,
    metrics: Arc<Metrics>,
}

impl GrpcServer {
    /// Create a new gRPC server
    pub fn new(config: Config, metrics: Arc<Metrics>) -> Self {
        Self { config, metrics }
    }

    /// Get the service instance
    pub fn create_service(&self) -> AmwajMediaService {
        AmwajMediaService::new(self.config.clone(), Arc::clone(&self.metrics))
    }

    /// Start the gRPC server
    pub async fn start(self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);

        tracing::info!("gRPC server starting on {}", addr);

        // For now, we use a TCP listener as a stub
        // Full tonic gRPC server will be added when proto compilation is integrated
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("Server listening on {}", addr);
        
        // Create service for validation
        let _service = self.create_service();
        
        loop {
            let (socket, peer_addr) = listener.accept().await?;
            let metrics = Arc::clone(&self.metrics);
            
            tokio::spawn(async move {
                tracing::debug!("New connection from {}", peer_addr);
                metrics.active_connections.inc();
                
                // Handle connection (stub for now)
                // Real gRPC handling would use tonic here
                drop(socket);
                
                metrics.active_connections.dec();
            });
        }
    }

    /// Start the server with graceful shutdown
    pub async fn start_with_shutdown(
        self,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);

        tracing::info!("gRPC server starting on {}", addr);

        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("Server listening on {} (with graceful shutdown)", addr);
        
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    let (socket, peer_addr) = accept_result?;
                    let metrics = Arc::clone(&self.metrics);
                    
                    tokio::spawn(async move {
                        tracing::debug!("New connection from {}", peer_addr);
                        metrics.active_connections.inc();
                        drop(socket);
                        metrics.active_connections.dec();
                    });
                }
                _ = &mut shutdown_rx => {
                    tracing::info!("Shutdown signal received, stopping server");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Get the server address
    pub fn address(&self) -> String {
        format!("{}:{}", self.config.server.host, self.config.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let config = Config::default();
        let metrics = Arc::new(Metrics::new(&config));
        let server = GrpcServer::new(config, metrics);
        
        assert_eq!(server.address(), "0.0.0.0:50051");
    }

    #[test]
    fn test_service_creation() {
        let config = Config::default();
        let metrics = Arc::new(Metrics::new(&config));
        let server = GrpcServer::new(config, metrics);
        
        let service = server.create_service();
        assert_eq!(service.config().server.port, 50051);
    }

    #[tokio::test]
    async fn test_server_graceful_shutdown() {
        let config = Config {
            server: crate::config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 50099, // Different port to avoid conflicts
                worker_threads: 1,
            },
            ..Config::default()
        };
        let metrics = Arc::new(Metrics::new(&config));
        let server = GrpcServer::new(config, metrics);
        
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        
        // Start server in background
        let handle = tokio::spawn(async move {
            server.start_with_shutdown(shutdown_rx).await
        });
        
        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Send shutdown signal
        let _ = shutdown_tx.send(());
        
        // Wait for server to stop
        let result = handle.await;
        assert!(result.is_ok());
    }
}
