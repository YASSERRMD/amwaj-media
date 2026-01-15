#[cfg(test)]
mod turn_detection_tests {
    use amwaj_media::detection::{TurnDetectionEngine, TurnDetectionConfig, TurnEvent, TurnState, MultiSignalFusion};
    use amwaj_media::audio::AudioFeatures;

    fn create_features(volume_db: f32) -> AudioFeatures {
        AudioFeatures {
            volume_db,
            pitch_hz: 200.0,
            spectral_centroid: 0.0,
            zero_crossing_rate: 0.0,
        }
    }

    #[test]
    fn test_turn_detection_speech_to_silence() {
        let config = TurnDetectionConfig {
            vad_threshold_enter: 0.6,
            vad_threshold_exit: 0.3,
            min_speech_duration_ms: 250,
            max_silence_duration_ms: 400,
            volume_threshold_db: -40.0,
        };

        let mut engine = TurnDetectionEngine::new(config);
        let features = create_features(-20.0);

        // Start speaking
        let event = engine.process(0.8, &features, 20);
        assert_eq!(event, TurnEvent::TurnStarted);

        // Continue speaking
        for _ in 0..15 {
            engine.process(0.8, &features, 20);
        }

        // Silence starts
        let features_silent = create_features(-50.0);
        let mut turn_ended = false;
        for _ in 0..25 {
            let event = engine.process(0.1, &features_silent, 20);
            if matches!(event, TurnEvent::TurnEnded(_)) {
                turn_ended = true;
                break;
            }
        }

        assert!(turn_ended, "Turn should have ended");
    }

    #[test]
    fn test_turn_detection_short_speech_ignored() {
        let config = TurnDetectionConfig {
            vad_threshold_enter: 0.6,
            vad_threshold_exit: 0.3,
            min_speech_duration_ms: 250,
            max_silence_duration_ms: 400,
            volume_threshold_db: -40.0,
        };

        let mut engine = TurnDetectionEngine::new(config);
        let features = create_features(-20.0);

        // Very brief speech (less than min duration)
        engine.process(0.8, &features, 20);
        engine.process(0.8, &features, 20);
        engine.process(0.8, &features, 20);

        // Silence
        let features_silent = create_features(-50.0);
        let mut received_turn_ended = false;
        for _ in 0..25 {
            let event = engine.process(0.1, &features_silent, 20);
            if matches!(event, TurnEvent::TurnEnded(_)) {
                received_turn_ended = true;
            }
        }

        // Short speech should be ignored
        assert!(!received_turn_ended, "Short speech should not trigger turn ended");
    }

    #[test]
    fn test_turn_detection_speech_resume() {
        let config = TurnDetectionConfig::default();
        let mut engine = TurnDetectionEngine::new(config);
        let features = create_features(-20.0);

        // Start speaking
        engine.process(0.8, &features, 20);
        for _ in 0..5 {
            engine.process(0.8, &features, 20);
        }

        // Brief pause
        for _ in 0..5 {
            engine.process(0.1, &features, 20);
        }
        
        // Resume speaking before silence threshold
        engine.process(0.8, &features, 20);
        
        assert_eq!(engine.state(), TurnState::Speaking);
    }

    #[test]
    fn test_multi_signal_fusion() {
        let fusion = MultiSignalFusion::new();
        let features = create_features(-20.0);

        let score = fusion.fuse_signals(0.8, &features, None);
        assert!(score > 0.5); // High confidence voice detected

        let score_silent = fusion.fuse_signals(0.1, &create_features(-60.0), None);
        assert!(score_silent < 0.3); // Low confidence in silence
    }

    #[test]
    fn test_multi_signal_context_aware() {
        let fusion = MultiSignalFusion::new();
        let features = create_features(-25.0);

        let score_neutral = fusion.fuse_signals(0.5, &features, None);
        let score_expecting = fusion.fuse_signals(0.5, &features, Some("expecting_response"));

        assert!(score_expecting > score_neutral);
    }

    #[test]
    fn test_engine_reset() {
        let mut engine = TurnDetectionEngine::new(TurnDetectionConfig::default());
        let features = create_features(-20.0);

        // Build up some state
        engine.process(0.8, &features, 20);
        engine.process(0.8, &features, 20);
        assert!(engine.speech_duration_ms() > 0);

        // Reset
        engine.reset();
        assert_eq!(engine.state(), TurnState::Idle);
        assert_eq!(engine.speech_duration_ms(), 0);
        assert_eq!(engine.silence_duration_ms(), 0);
    }

    #[test]
    fn test_average_vad() {
        let mut engine = TurnDetectionEngine::new(TurnDetectionConfig::default());
        let features = create_features(-20.0);

        // Process several frames
        for _ in 0..5 {
            engine.process(0.8, &features, 20);
        }
        for _ in 0..5 {
            engine.process(0.2, &features, 20);
        }

        let avg = engine.average_vad();
        assert!(avg > 0.4 && avg < 0.6); // Average of 0.8 and 0.2
    }
}
