#[cfg(test)]
mod webrtc_tests {
    use amwaj_media::webrtc::{JitterBuffer, OpusDecoder, RtpPacket, WebRtcManager};

    #[test]
    fn test_rtp_packet_parsing() {
        let data = vec![
            0x80, 0x78, 0x00, 0x01, // Version, marker, sequence
            0x00, 0x00, 0x00, 0x00, // Timestamp
            0x00, 0x00, 0x00, 0x01, // SSRC
            0xAA, 0xBB, 0xCC, 0xDD, // Payload
        ];

        let packet = RtpPacket::parse(&data).expect("Failed to parse RTP packet");
        assert_eq!(packet.version, 2);
        assert_eq!(packet.sequence_number, 1);
        assert_eq!(packet.payload.len(), 4);
    }

    #[test]
    fn test_rtp_packet_with_marker() {
        let data = vec![
            0x80, 0xF8, 0x00, 0x10, // Version, marker=1, PT=120, seq=16
            0x00, 0x01, 0x00, 0x00, // Timestamp
            0xDE, 0xAD, 0xBE, 0xEF, // SSRC
            0x01, 0x02, // Payload
        ];

        let packet = RtpPacket::parse(&data).expect("Failed to parse");
        assert!(packet.marker);
        assert_eq!(packet.payload_type, 120);
        assert_eq!(packet.sequence_number, 16);
        assert_eq!(packet.ssrc, 0xDEADBEEF);
    }

    #[test]
    fn test_jitter_buffer_insert_retrieve() {
        let mut buffer = JitterBuffer::new(100, 16000);

        let data = vec![0x01, 0x02, 0x03];
        buffer.insert(100, data.clone());

        let retrieved = buffer.get_ready_frame();
        assert_eq!(retrieved, Some(data));
    }

    #[test]
    fn test_jitter_buffer_ordering() {
        let mut buffer = JitterBuffer::new(100, 16000);

        buffer.insert(102, vec![3]);
        buffer.insert(100, vec![1]);
        buffer.insert(101, vec![2]);

        assert_eq!(buffer.get_ready_frame(), Some(vec![1]));
        assert_eq!(buffer.get_ready_frame(), Some(vec![2]));
        assert_eq!(buffer.get_ready_frame(), Some(vec![3]));
    }

    #[test]
    fn test_jitter_buffer_get_multiple() {
        let mut buffer = JitterBuffer::new(100, 16000);

        buffer.insert(1, vec![1]);
        buffer.insert(2, vec![2]);
        buffer.insert(3, vec![3]);

        let frames = buffer.get_ready_frames(2);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], vec![1]);
        assert_eq!(frames[1], vec![2]);
    }

    #[test]
    fn test_opus_decoder_creation() {
        let decoder = OpusDecoder::new(16000);
        assert_eq!(decoder.sample_rate(), 16000);
        assert_eq!(decoder.frames_decoded(), 0);
    }

    #[test]
    fn test_opus_decode_stub() {
        let mut decoder = OpusDecoder::new(16000);
        let result = decoder.decode(&[0xFF; 100]);
        assert!(result.is_ok());

        let pcm = result.unwrap();
        assert!(!pcm.is_empty());
        assert_eq!(decoder.frames_decoded(), 1);
    }

    #[test]
    fn test_webrtc_manager() {
        let mut manager = WebRtcManager::new();

        assert!(manager.create_connection("session1".to_string()).is_ok());
        assert!(manager.create_connection("session2".to_string()).is_ok());

        assert_eq!(manager.connection_count(), 2);

        let conn = manager.get_connection("session1");
        assert!(conn.is_ok());
        assert_eq!(conn.unwrap().session_id(), "session1");
    }

    #[test]
    fn test_webrtc_manager_remove() {
        let mut manager = WebRtcManager::new();

        manager.create_connection("session1".to_string()).unwrap();
        assert_eq!(manager.connection_count(), 1);

        let removed = manager.remove_connection("session1");
        assert!(removed.is_some());
        assert_eq!(manager.connection_count(), 0);
    }
}
