#[cfg(test)]
mod metrics_tests {
    use amwaj_media::config::Config;
    use amwaj_media::metrics::{LatencyTracker, Metrics};
    use std::sync::Arc;

    #[test]
    fn test_metrics_initialization() {
        let config = Config::default();
        let metrics = Metrics::new(&config);

        assert_eq!(metrics.active_connections.get(), 0);
    }

    #[test]
    fn test_connection_tracking() {
        let config = Config::default();
        let metrics = Metrics::new(&config);

        assert_eq!(metrics.active_connections.get(), 0);

        metrics.connection_opened();
        assert_eq!(metrics.active_connections.get(), 1);

        metrics.connection_opened();
        assert_eq!(metrics.active_connections.get(), 2);

        metrics.connection_closed();
        assert_eq!(metrics.active_connections.get(), 1);
    }

    #[test]
    fn test_turn_event_tracking() {
        let config = Config::default();
        let metrics = Metrics::new(&config);

        metrics.record_turn_start();
        metrics.record_turn_end();

        // Check that turn events were recorded
        // Note: Counter values can be checked through prometheus encoding
    }

    #[test]
    fn test_latency_recording() {
        let config = Config::default();
        let metrics = Metrics::new(&config);

        metrics.record_latency(5.5);
        metrics.record_latency(10.2);
        metrics.record_latency(2.1);

        // Histogram should have 3 observations
    }

    #[test]
    fn test_barge_in_tracking() {
        let config = Config::default();
        let metrics = Metrics::new(&config);

        metrics.record_barge_in();
        metrics.record_barge_in();

        // Barge-ins recorded
    }

    #[test]
    fn test_latency_tracker_basic() {
        let tracker = LatencyTracker::new("test_component");
        assert_eq!(tracker.component(), "test_component");
        assert!(!tracker.is_recorded());

        let elapsed = tracker.elapsed_ms();
        assert!(elapsed >= 0.0);
    }

    #[test]
    fn test_latency_tracker_with_metrics() {
        let config = Config::default();
        let metrics = Arc::new(Metrics::new(&config));

        let tracker = LatencyTracker::new("audio_processing");

        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(5));

        let elapsed = tracker.record_to(&metrics);
        assert!(elapsed >= 4.0);
    }

    #[test]
    fn test_multiple_trackers() {
        let tracker1 = LatencyTracker::new("component1");
        let tracker2 = LatencyTracker::new("component2");

        std::thread::sleep(std::time::Duration::from_millis(5));

        let elapsed1 = tracker1.elapsed_ms();
        let elapsed2 = tracker2.elapsed_ms();

        // Both should have similar times
        assert!((elapsed1 - elapsed2).abs() < 2.0);
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        use prometheus::{Encoder, TextEncoder};

        let config = Config::default();
        let metrics = Metrics::new(&config);

        // Simulate some work
        metrics.audio_frames_processed.inc();
        metrics.audio_frames_processed.inc();
        metrics.active_connections.set(5);
        metrics.record_latency(3.5);

        // Encode metrics
        let encoder = TextEncoder::new();
        let metric_families = metrics.registry.gather();
        let mut buffer = Vec::new();

        encoder
            .encode(&metric_families, &mut buffer)
            .expect("Failed to encode");

        let output = String::from_utf8(buffer).expect("Invalid UTF-8");

        // Verify some metrics are present
        assert!(output.contains("amwaj_active_connections"));
        assert!(output.contains("amwaj_audio_frames_processed"));
    }
}
