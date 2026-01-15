//! Configuration management for Amwaj Media Server

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub grpc: GrpcConfig,
    pub webrtc: WebRtcConfig,
    pub audio: AudioConfig,
    pub detection: DetectionConfig,
    pub metrics: MetricsConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcConfig {
    pub max_message_size: usize,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcConfig {
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u32,
    pub frame_duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    pub vad_sensitivity: f32,
    pub min_turn_duration_ms: u32,
    pub max_silence_duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub prometheus_port: u16,
    pub enable_jaeger_tracing: bool,
    pub jaeger_agent_host: String,
    pub jaeger_agent_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Config {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_env() -> Self {
        Self::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 50051,
                worker_threads: num_cpus::get(),
            },
            grpc: GrpcConfig {
                max_message_size: 10 * 1024 * 1024,
                timeout_secs: 30,
            },
            webrtc: WebRtcConfig {
                stun_servers: vec!["stun:stun.l.google.com:19302".to_string()],
                turn_servers: vec![],
            },
            audio: AudioConfig {
                sample_rate: 16000,
                channels: 1,
                frame_duration_ms: 20,
            },
            detection: DetectionConfig {
                vad_sensitivity: 0.6,
                min_turn_duration_ms: 250,
                max_silence_duration_ms: 400,
            },
            metrics: MetricsConfig {
                prometheus_port: 9090,
                enable_jaeger_tracing: true,
                jaeger_agent_host: "localhost".to_string(),
                jaeger_agent_port: 6831,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }
}
