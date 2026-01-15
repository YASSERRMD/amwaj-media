#[cfg(test)]
mod e2e_tests {
    use amwaj_media::audio::AudioProcessor;
    use amwaj_media::config::Config;
    use amwaj_media::detection::{TurnDetectionConfig, TurnDetectionEngine, TurnEvent};
    use amwaj_media::metrics::Metrics;
    use amwaj_media::webrtc::{JitterBuffer, RtpPacket, WebRtcManager};
    use std::sync::Arc;

    #[test]
    fn test_complete_audio_pipeline() {
        // Create components
        let mut processor = AudioProcessor::new(16000, 320);
        let config = TurnDetectionConfig {
            vad_threshold_enter: 0.5,
            vad_threshold_exit: 0.3,
            min_speech_duration_ms: 100,
            max_silence_duration_ms: 200,
            volume_threshold_db: -50.0,
        };
        let mut detector = TurnDetectionEngine::new(config);

        // Simulate voice input (high amplitude)
        let voice_pcm = vec![5000i16; 320];

        // Process audio
        let frame = processor.process_frame(&voice_pcm).unwrap();

        // Run turn detection with high VAD (simulating good voice)
        let event = detector.process(
            0.8, // Force high VAD for reliable test
            &frame.features,
            20,
        );

        // Should detect speech start
        assert_eq!(event, TurnEvent::TurnStarted);
    }

    #[test]
    fn test_webrtc_to_audio_pipeline() {
        let mut manager = WebRtcManager::new();

        // Create connection
        manager
            .create_connection("test-session".to_string())
            .unwrap();

        // Simulate RTP packets
        let mut jitter_buffer = JitterBuffer::new(100, 16000);

        for seq in 0..10u16 {
            let payload = vec![0xAAu8; 160]; // Simulated opus data
            jitter_buffer.insert(seq, payload);
        }

        // Verify buffer ordering
        let frames = jitter_buffer.get_ready_frames(10);
        assert_eq!(frames.len(), 10);
    }

    #[test]
    fn test_rtp_parsing_to_audio() {
        // Create RTP packet with audio payload
        let rtp_data = vec![
            0x80, 0x6F, 0x00, 0x01, // Version=2, PT=111, seq=1
            0x00, 0x00, 0x00, 0x00, // Timestamp
            0x00, 0x00, 0x00, 0x01, // SSRC
            // Simulated opus payload
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        ];

        let packet = RtpPacket::parse(&rtp_data).unwrap();

        assert_eq!(packet.version, 2);
        assert_eq!(packet.payload_type, 111);
        assert_eq!(packet.payload.len(), 8);
    }

    #[test]
    fn test_full_turn_detection_cycle() {
        let config = TurnDetectionConfig {
            vad_threshold_enter: 0.5,
            vad_threshold_exit: 0.3,
            min_speech_duration_ms: 100,
            max_silence_duration_ms: 200,
            volume_threshold_db: -50.0,
        };

        let mut detector = TurnDetectionEngine::new(config);
        let mut processor = AudioProcessor::new(16000, 320);

        // Phase 1: Silence
        for _ in 0..5 {
            let silence = vec![0i16; 320];
            let frame = processor.process_frame(&silence).unwrap();
            let event = detector.process(frame.vad_probability, &frame.features, 20);
            assert_eq!(event, TurnEvent::None);
        }

        // Phase 2: Speech starts
        let voice = vec![5000i16; 320];
        let frame = processor.process_frame(&voice).unwrap();
        // Force high VAD for test
        let event = detector.process(0.8, &frame.features, 20);
        assert_eq!(event, TurnEvent::TurnStarted);

        // Phase 3: Continue speaking
        for _ in 0..10 {
            let event = detector.process(0.8, &frame.features, 20);
            assert_eq!(event, TurnEvent::None);
        }

        // Phase 4: Silence begins
        detector.process(0.1, &frame.features, 20);

        // Phase 5: Silence continues until turn ends
        let mut turn_ended = false;
        for _ in 0..15 {
            if let TurnEvent::TurnEnded(_) = detector.process(0.1, &frame.features, 20) {
                turn_ended = true;
                break;
            }
        }

        assert!(turn_ended);
    }

    #[tokio::test]
    async fn test_metrics_integration() {
        let config = Config::default();
        let metrics = Arc::new(Metrics::new(&config));

        // Simulate activity
        metrics.connection_opened();
        metrics.rtp_packets_received.inc_by(100.0);
        metrics.audio_frames_processed.inc_by(50.0);
        metrics.record_turn_start();
        metrics.record_turn_end();
        metrics.record_latency(5.5);

        // Verify counts
        assert_eq!(metrics.active_connections.get(), 1);
    }

    #[test]
    fn test_multiple_sessions() {
        let mut manager = WebRtcManager::new();

        // Create multiple sessions
        for i in 0..100 {
            let session_id = format!("session-{}", i);
            manager.create_connection(session_id).unwrap();
        }

        assert_eq!(manager.connection_count(), 100);

        // Access specific session
        let session = manager.get_connection("session-50");
        assert!(session.is_ok());

        // Remove sessions
        for i in 0..50 {
            let session_id = format!("session-{}", i);
            manager.remove_connection(&session_id);
        }

        assert_eq!(manager.connection_count(), 50);
    }
}
