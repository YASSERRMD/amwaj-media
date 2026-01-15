#[cfg(test)]
mod tests {
    use amwaj_media::config::Config;
    use amwaj_media::metrics::Metrics;
    
    #[test]
    fn test_config_load_default() {
        let config = Config::default();
        assert_eq!(config.server.port, 50051);
        assert_eq!(config.audio.sample_rate, 16000);
    }

    #[test]
    fn test_config_values_valid() {
        let config = Config::default();
        assert!(config.detection.vad_sensitivity > 0.0);
        assert!(config.detection.vad_sensitivity <= 1.0);
    }

    #[tokio::test]
    async fn test_server_initialization() {
        let config = Config::default();
        let metrics = Metrics::new(&config);
        assert_eq!(metrics.active_connections.get(), 0);
    }

    #[test]
    fn test_metrics_creation() {
        let config = Config::default();
        let metrics = Metrics::new(&config);
        // Counter starts at 0
        assert_eq!(metrics.active_connections.get(), 0);
    }

    #[test]
    fn test_audio_config_defaults() {
        let config = Config::default();
        assert_eq!(config.audio.sample_rate, 16000);
        assert_eq!(config.audio.channels, 1);
        assert_eq!(config.audio.frame_duration_ms, 20);
    }

    #[test]
    fn test_detection_config_defaults() {
        let config = Config::default();
        assert_eq!(config.detection.min_turn_duration_ms, 250);
        assert_eq!(config.detection.max_silence_duration_ms, 400);
    }
}
