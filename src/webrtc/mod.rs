//! WebRTC module for Amwaj Media Server

pub mod codec;
pub mod jitter_buffer;
pub mod peer_connection;
pub mod rtp_handler;

pub use codec::OpusDecoder;
pub use jitter_buffer::JitterBuffer;
pub use peer_connection::PeerConnection;
pub use rtp_handler::RtpPacket;

use std::collections::HashMap;

pub struct WebRtcManager {
    connections: HashMap<String, PeerConnection>,
}

impl WebRtcManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    pub fn create_connection(&mut self, session_id: String) -> anyhow::Result<()> {
        let peer = PeerConnection::new(session_id.clone());
        self.connections.insert(session_id, peer);
        Ok(())
    }

    pub fn get_connection(&mut self, session_id: &str) -> anyhow::Result<&mut PeerConnection> {
        self.connections
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))
    }

    pub fn remove_connection(&mut self, session_id: &str) -> Option<PeerConnection> {
        self.connections.remove(session_id)
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

impl Default for WebRtcManager {
    fn default() -> Self {
        Self::new()
    }
}
