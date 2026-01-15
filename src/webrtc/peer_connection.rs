//! WebRTC Peer Connection Handler

use crate::webrtc::{JitterBuffer, OpusDecoder, RtpPacket};
use parking_lot::Mutex;
use std::sync::Arc;

/// Represents a WebRTC peer connection
pub struct PeerConnection {
    session_id: String,
    is_connected: bool,
    remote_sdp: Option<String>,
    local_sdp: Option<String>,
    jitter_buffer: Arc<Mutex<JitterBuffer>>,
    decoder: OpusDecoder,
    packets_processed: u64,
}

impl PeerConnection {
    /// Create a new peer connection
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            is_connected: false,
            remote_sdp: None,
            local_sdp: None,
            jitter_buffer: Arc::new(Mutex::new(JitterBuffer::new(100, 16000))),
            decoder: OpusDecoder::new(16000),
            packets_processed: 0,
        }
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Set connected state
    pub fn set_connected(&mut self, connected: bool) {
        self.is_connected = connected;
    }

    /// Set remote SDP offer
    pub fn set_remote_sdp(&mut self, sdp: String) -> anyhow::Result<()> {
        self.remote_sdp = Some(sdp);
        Ok(())
    }

    /// Get remote SDP
    pub fn remote_sdp(&self) -> Option<&String> {
        self.remote_sdp.as_ref()
    }

    /// Create SDP answer
    pub fn create_answer(&mut self) -> anyhow::Result<String> {
        // TODO: Implement proper SDP answer creation
        let answer = format!(
            "v=0\r\n\
             o=- 0 0 IN IP4 127.0.0.1\r\n\
             s=Amwaj Media Server\r\n\
             t=0 0\r\n\
             m=audio 0 RTP/AVP 111\r\n\
             a=rtpmap:111 opus/48000/2\r\n"
        );
        self.local_sdp = Some(answer.clone());
        Ok(answer)
    }

    /// Handle incoming RTP packet
    pub fn on_rtp_packet(&mut self, packet_data: &[u8]) -> anyhow::Result<Option<Vec<i16>>> {
        let packet = RtpPacket::parse(packet_data)?;

        self.packets_processed += 1;

        // Insert into jitter buffer
        {
            let mut buffer = self.jitter_buffer.lock();
            buffer.insert(packet.sequence_number, packet.payload.clone());
        }

        // Try to get a ready frame and decode it
        let frame = {
            let mut buffer = self.jitter_buffer.lock();
            buffer.get_ready_frame()
        };

        if let Some(opus_data) = frame {
            let pcm = self.decoder.decode(&opus_data)?;
            Ok(Some(pcm))
        } else {
            Ok(None)
        }
    }

    /// Get jitter buffer statistics
    pub fn get_buffer_stats(&self) -> BufferStats {
        let buffer = self.jitter_buffer.lock();
        BufferStats {
            size: buffer.size(),
            level_percent: buffer.level_percent(),
            packet_loss_ratio: buffer.packet_loss_ratio(),
        }
    }

    /// Get total packets processed
    pub fn packets_processed(&self) -> u64 {
        self.packets_processed
    }

    /// Clear the jitter buffer
    pub fn clear_buffer(&mut self) {
        let mut buffer = self.jitter_buffer.lock();
        buffer.clear();
    }
}

/// Buffer statistics
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub size: usize,
    pub level_percent: f32,
    pub packet_loss_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_connection_creation() {
        let peer = PeerConnection::new("test-session".to_string());
        assert_eq!(peer.session_id(), "test-session");
        assert!(!peer.is_connected());
    }

    #[test]
    fn test_set_remote_sdp() {
        let mut peer = PeerConnection::new("test".to_string());
        let sdp = "v=0\r\no=...".to_string();

        assert!(peer.set_remote_sdp(sdp.clone()).is_ok());
        assert_eq!(peer.remote_sdp(), Some(&sdp));
    }

    #[test]
    fn test_create_answer() {
        let mut peer = PeerConnection::new("test".to_string());
        let answer = peer.create_answer();

        assert!(answer.is_ok());
        let answer_str = answer.unwrap();
        assert!(answer_str.contains("v=0"));
        assert!(answer_str.contains("opus"));
    }

    #[test]
    fn test_rtp_packet_handling() {
        let mut peer = PeerConnection::new("test".to_string());

        // Create a valid RTP packet
        let rtp_data = vec![
            0x80, 0x6F, 0x00, 0x01, // Version=2, PT=111 (opus), seq=1
            0x00, 0x00, 0x00, 0x00, // Timestamp
            0x00, 0x00, 0x00, 0x01, // SSRC
            0xAA, 0xBB, 0xCC, 0xDD, // Payload (opus data)
        ];

        let result = peer.on_rtp_packet(&rtp_data);
        assert!(result.is_ok());
        assert_eq!(peer.packets_processed(), 1);
    }
}
