//! Metrics infrastructure for Amwaj Media Server

pub mod prometheus;

use ::prometheus::{Counter, Histogram, HistogramOpts, IntGauge, Registry};
use crate::config::Config;

pub struct Metrics {
    pub registry: Registry,
    pub active_connections: IntGauge,
    pub rtp_packets_received: Counter,
    pub audio_frames_processed: Counter,
    pub turn_events_detected: Counter,
    pub processing_latency_ms: Histogram,
    pub grpc_messages_sent: Counter,
}

impl Metrics {
    pub fn new(_config: &Config) -> Self {
        let registry = Registry::new();
        
        let active_connections = IntGauge::new(
            "amwaj_active_connections",
            "Number of active WebRTC connections"
        ).expect("Failed to create metric");
        
        let rtp_packets_received = Counter::new(
            "amwaj_rtp_packets_received_total",
            "Total RTP packets received"
        ).expect("Failed to create metric");
        
        let audio_frames_processed = Counter::new(
            "amwaj_audio_frames_processed_total",
            "Total audio frames processed"
        ).expect("Failed to create metric");
        
        let turn_events_detected = Counter::new(
            "amwaj_turn_events_detected_total",
            "Total turn events detected"
        ).expect("Failed to create metric");
        
        let processing_latency_opts = HistogramOpts::new(
            "amwaj_processing_latency_ms",
            "Processing latency in milliseconds"
        );
        let processing_latency_ms = Histogram::with_opts(processing_latency_opts)
            .expect("Failed to create metric");
        
        let grpc_messages_sent = Counter::new(
            "amwaj_grpc_messages_sent_total",
            "Total gRPC messages sent"
        ).expect("Failed to create metric");
        
        // Register all metrics
        registry.register(Box::new(active_connections.clone())).unwrap();
        registry.register(Box::new(rtp_packets_received.clone())).unwrap();
        registry.register(Box::new(audio_frames_processed.clone())).unwrap();
        registry.register(Box::new(turn_events_detected.clone())).unwrap();
        registry.register(Box::new(processing_latency_ms.clone())).unwrap();
        registry.register(Box::new(grpc_messages_sent.clone())).unwrap();
        
        Self {
            registry,
            active_connections,
            rtp_packets_received,
            audio_frames_processed,
            turn_events_detected,
            processing_latency_ms,
            grpc_messages_sent,
        }
    }
}
