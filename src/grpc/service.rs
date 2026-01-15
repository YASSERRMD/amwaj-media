//! gRPC Service Implementation

use std::sync::Arc;
use tokio::sync::mpsc;
use crate::config::Config;
use crate::metrics::Metrics;

/// gRPC Media Service handler
pub struct AmwajMediaService {
    config: Arc<Config>,
    metrics: Arc<Metrics>,
}

impl AmwajMediaService {
    /// Create a new AmwajMediaService
    pub fn new(config: Config, metrics: Arc<Metrics>) -> Self {
        Self {
            config: Arc::new(config),
            metrics,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the metrics
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}

/// Media event types for the gRPC stream
#[derive(Debug, Clone)]
pub enum MediaEvent {
    AudioFrame {
        session_id: String,
        timestamp_ms: i64,
        pcm_data: Vec<u8>,
        sample_rate: u32,
        channels: u32,
    },
    TurnStarted {
        session_id: String,
        timestamp_ms: i64,
        vad_probability: f32,
    },
    TurnEnded {
        session_id: String,
        timestamp_ms: i64,
        duration_ms: u32,
    },
    PartialTranscript {
        session_id: String,
        timestamp_ms: i64,
        text: String,
        confidence: f32,
    },
    SessionEnded {
        session_id: String,
        duration_ms: i64,
        total_frames: u32,
    },
}

/// Orchestration commands from the server
#[derive(Debug, Clone)]
pub enum OrchestrationCommand {
    PlayAudio {
        session_id: String,
        audio_data: Vec<u8>,
        audio_format: String,
    },
    StopAudio {
        session_id: String,
        reason: String,
    },
    ClearContext {
        session_id: String,
        context_type: String,
    },
    AdjustVAD {
        session_id: String,
        sensitivity: f32,
        threshold_ms: u32,
    },
}

/// Session handler for managing a single media stream session
pub struct SessionHandler {
    session_id: String,
    event_tx: mpsc::Sender<MediaEvent>,
    command_rx: mpsc::Receiver<OrchestrationCommand>,
    #[allow(dead_code)]
    config: Arc<Config>,
    metrics: Arc<Metrics>,
}

impl SessionHandler {
    /// Create a new session handler
    pub fn new(
        session_id: String,
        config: Arc<Config>,
        metrics: Arc<Metrics>,
    ) -> (Self, mpsc::Receiver<MediaEvent>, mpsc::Sender<OrchestrationCommand>) {
        let (event_tx, event_rx) = mpsc::channel(100);
        let (command_tx, command_rx) = mpsc::channel(100);

        let handler = Self {
            session_id,
            event_tx,
            command_rx,
            config,
            metrics,
        };

        (handler, event_rx, command_tx)
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Send a media event
    pub async fn send_event(&self, event: MediaEvent) -> anyhow::Result<()> {
        self.event_tx
            .send(event)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send event: {}", e))?;
        self.metrics.grpc_messages_sent.inc();
        Ok(())
    }

    /// Receive the next orchestration command
    pub async fn receive_command(&mut self) -> Option<OrchestrationCommand> {
        self.command_rx.recv().await
    }

    /// Check if the event channel is closed
    pub fn is_closed(&self) -> bool {
        self.event_tx.is_closed()
    }
}

/// Message buffer for handling backpressure
pub struct MessageBuffer<T> {
    buffer: Vec<T>,
    max_size: usize,
}

impl<T> MessageBuffer<T> {
    /// Create a new message buffer
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Add a message to the buffer
    pub fn push(&mut self, msg: T) -> bool {
        if self.buffer.len() < self.max_size {
            self.buffer.push(msg);
            true
        } else {
            false // Buffer full, backpressure
        }
    }

    /// Take all messages from the buffer
    pub fn drain(&mut self) -> Vec<T> {
        std::mem::take(&mut self.buffer)
    }

    /// Get current buffer size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.max_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let config = Config::default();
        let metrics = Arc::new(Metrics::new(&config));
        let service = AmwajMediaService::new(config, metrics);
        
        assert_eq!(service.config().server.port, 50051);
    }

    #[tokio::test]
    async fn test_session_handler() {
        let config = Arc::new(Config::default());
        let metrics = Arc::new(Metrics::new(&config));
        
        let (handler, mut event_rx, _command_tx) = SessionHandler::new(
            "test-session".to_string(),
            config,
            metrics,
        );
        
        assert_eq!(handler.session_id(), "test-session");
        
        // Send an event
        let event = MediaEvent::TurnStarted {
            session_id: "test-session".to_string(),
            timestamp_ms: 1000,
            vad_probability: 0.8,
        };
        
        handler.send_event(event).await.unwrap();
        
        // Receive the event
        let received = event_rx.recv().await;
        assert!(received.is_some());
    }

    #[test]
    fn test_message_buffer() {
        let mut buffer: MessageBuffer<i32> = MessageBuffer::new(3);
        
        assert!(buffer.push(1));
        assert!(buffer.push(2));
        assert!(buffer.push(3));
        assert!(!buffer.push(4)); // Buffer full
        
        assert!(buffer.is_full());
        assert_eq!(buffer.len(), 3);
        
        let drained = buffer.drain();
        assert_eq!(drained, vec![1, 2, 3]);
        assert!(buffer.is_empty());
    }
}
