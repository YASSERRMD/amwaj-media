//! Error types for Amwaj Media Server

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AmwajError {
    #[error("WebRTC error: {0}")]
    WebRtcError(String),

    #[error("Audio processing error: {0}")]
    AudioError(String),

    #[error("gRPC error: {0}")]
    GrpcError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Turn detection error: {0}")]
    DetectionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, AmwajError>;
