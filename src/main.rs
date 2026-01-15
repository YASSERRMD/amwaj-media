//! Amwaj Media Server - Real-time media server for voice agents

use amwaj_media::{config::Config, grpc::server::GrpcServer, metrics::Metrics};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(Parser)]
#[command(name = "Amwaj Media Server")]
#[command(about = "Real-time media server for voice agents")]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = Config::from_file(&args.config).unwrap_or_else(|_| Config::default());

    // Initialize logging
    initialize_logging(&config);

    // Initialize metrics
    let metrics = Arc::new(Metrics::new(&config));

    // Start Prometheus metrics server
    let metrics_addr = format!("0.0.0.0:{}", config.metrics.prometheus_port).parse()?;
    let metrics_registry = metrics.registry.clone();
    tokio::spawn(async move {
        if let Err(e) =
            amwaj_media::metrics::prometheus::start_metrics_server(metrics_addr, metrics_registry)
                .await
        {
            tracing::error!("Metrics server error: {}", e);
        }
    });

    // Create and start gRPC server
    let grpc_server = GrpcServer::new(config.clone(), metrics);

    info!(
        "Starting Amwaj Media Server on {}:{}",
        config.server.host, config.server.port
    );

    grpc_server.start().await?;

    Ok(())
}

fn initialize_logging(config: &Config) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.logging.level));

    if config.logging.format == "json" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer())
            .init();
    }
}
