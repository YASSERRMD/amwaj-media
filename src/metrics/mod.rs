//! Metrics infrastructure for Amwaj Media Server

pub mod prometheus;
pub mod latency_tracker;

use ::prometheus::{Counter, Histogram, HistogramOpts, IntGauge, Registry};
use crate::config::Config;

/// Centralized metrics collection
pub struct Metrics {
    pub registry: Registry,
    pub active_connections: IntGauge,
    pub rtp_packets_received: Counter,
    pub audio_frames_processed: Counter,
    pub turn_events_detected: Counter,
    pub processing_latency_ms: Histogram,
    pub grpc_messages_sent: Counter,
    pub grpc_messages_received: Counter,
    pub vad_detections: Counter,
    pub turn_starts: Counter,
    pub turn_ends: Counter,
    pub barge_ins: Counter,
}

impl Metrics {
    /// Create a new Metrics instance
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
        ).buckets(vec![0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0]);
        let processing_latency_ms = Histogram::with_opts(processing_latency_opts)
            .expect("Failed to create metric");
        
        let grpc_messages_sent = Counter::new(
            "amwaj_grpc_messages_sent_total",
            "Total gRPC messages sent"
        ).expect("Failed to create metric");
        
        let grpc_messages_received = Counter::new(
            "amwaj_grpc_messages_received_total",
            "Total gRPC messages received"
        ).expect("Failed to create metric");
        
        let vad_detections = Counter::new(
            "amwaj_vad_detections_total",
            "Total voice activity detections"
        ).expect("Failed to create metric");
        
        let turn_starts = Counter::new(
            "amwaj_turn_starts_total",
            "Total turn start events"
        ).expect("Failed to create metric");
        
        let turn_ends = Counter::new(
            "amwaj_turn_ends_total",
            "Total turn end events"
        ).expect("Failed to create metric");
        
        let barge_ins = Counter::new(
            "amwaj_barge_ins_total",
            "Total barge-in events detected"
        ).expect("Failed to create metric");
        
        // Register all metrics
        registry.register(Box::new(active_connections.clone())).unwrap();
        registry.register(Box::new(rtp_packets_received.clone())).unwrap();
        registry.register(Box::new(audio_frames_processed.clone())).unwrap();
        registry.register(Box::new(turn_events_detected.clone())).unwrap();
        registry.register(Box::new(processing_latency_ms.clone())).unwrap();
        registry.register(Box::new(grpc_messages_sent.clone())).unwrap();
        registry.register(Box::new(grpc_messages_received.clone())).unwrap();
        registry.register(Box::new(vad_detections.clone())).unwrap();
        registry.register(Box::new(turn_starts.clone())).unwrap();
        registry.register(Box::new(turn_ends.clone())).unwrap();
        registry.register(Box::new(barge_ins.clone())).unwrap();
        
        Self {
            registry,
            active_connections,
            rtp_packets_received,
            audio_frames_processed,
            turn_events_detected,
            processing_latency_ms,
            grpc_messages_sent,
            grpc_messages_received,
            vad_detections,
            turn_starts,
            turn_ends,
            barge_ins,
        }
    }

    /// Record processing latency
    pub fn record_latency(&self, latency_ms: f64) {
        self.processing_latency_ms.observe(latency_ms);
    }

    /// Increment connection count
    pub fn connection_opened(&self) {
        self.active_connections.inc();
    }

    /// Decrement connection count
    pub fn connection_closed(&self) {
        self.active_connections.dec();
    }

    /// Record turn start
    pub fn record_turn_start(&self) {
        self.turn_starts.inc();
        self.turn_events_detected.inc();
    }

    /// Record turn end
    pub fn record_turn_end(&self) {
        self.turn_ends.inc();
        self.turn_events_detected.inc();
    }

    /// Record barge-in
    pub fn record_barge_in(&self) {
        self.barge_ins.inc();
    }
}

pub use latency_tracker::LatencyTracker;
