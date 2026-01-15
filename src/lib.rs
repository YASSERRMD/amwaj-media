//! Amwaj Media Server Library
//!
//! Real-time media server for voice agents using Rust, WebRTC, and gRPC.
//!
//! # Overview
//!
//! Amwaj Media Server provides:
//! - WebRTC streaming with RTP packet handling
//! - Audio processing with VAD and feature extraction
//! - Turn detection for conversational AI
//! - gRPC bidirectional streaming
//! - Prometheus metrics and latency tracking
//!
//! # Example
//!
//! ```rust,ignore
//! use amwaj_media::{Config, grpc::server::GrpcServer, metrics::Metrics};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::default();
//!     let metrics = Arc::new(Metrics::new(&config));
//!     
//!     let server = GrpcServer::new(config, metrics);
//!     server.start().await
//! }
//! ```

pub mod audio;
pub mod config;
pub mod detection;
pub mod error;
pub mod grpc;
pub mod metrics;
pub mod webrtc;

pub use config::Config;
pub use error::{AmwajError, Result};
