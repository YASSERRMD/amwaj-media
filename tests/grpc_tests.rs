#[cfg(test)]
mod grpc_tests {
    use amwaj_media::config::Config;
    use amwaj_media::metrics::Metrics;
    use amwaj_media::grpc::service::{AmwajMediaService, SessionHandler, MediaEvent, MessageBuffer};
    use amwaj_media::grpc::server::GrpcServer;
    use std::sync::Arc;

    #[test]
    fn test_grpc_service_creation() {
        let config = Config::default();
        let metrics = Arc::new(Metrics::new(&config));
        let service = AmwajMediaService::new(config, metrics);
        
        assert_eq!(service.config().server.port, 50051);
    }

    #[test]
    fn test_grpc_server_address() {
        let config = Config::default();
        let metrics = Arc::new(Metrics::new(&config));
        let server = GrpcServer::new(config, metrics);
        
        assert_eq!(server.address(), "0.0.0.0:50051");
    }

    #[tokio::test]
    async fn test_session_handler_creation() {
        let config = Arc::new(Config::default());
        let metrics = Arc::new(Metrics::new(&config));
        
        let (handler, _event_rx, _command_tx) = SessionHandler::new(
            "test-session".to_string(),
            config,
            metrics,
        );
        
        assert_eq!(handler.session_id(), "test-session");
        assert!(!handler.is_closed());
    }

    #[tokio::test]
    async fn test_send_receive_events() {
        let config = Arc::new(Config::default());
        let metrics = Arc::new(Metrics::new(&config));
        
        let (handler, mut event_rx, _command_tx) = SessionHandler::new(
            "test".to_string(),
            config,
            metrics,
        );

        // Send multiple events
        handler.send_event(MediaEvent::TurnStarted {
            session_id: "test".to_string(),
            timestamp_ms: 1000,
            vad_probability: 0.8,
        }).await.unwrap();

        handler.send_event(MediaEvent::TurnEnded {
            session_id: "test".to_string(),
            timestamp_ms: 2000,
            duration_ms: 1000,
        }).await.unwrap();

        // Receive events
        let event1 = event_rx.recv().await;
        let event2 = event_rx.recv().await;
        
        assert!(event1.is_some());
        assert!(event2.is_some());
    }

    #[test]
    fn test_message_buffer_operations() {
        let mut buffer: MessageBuffer<String> = MessageBuffer::new(5);
        
        // Buffer should start empty
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        
        // Add items
        for i in 0..5 {
            assert!(buffer.push(format!("msg-{}", i)));
        }
        
        // Buffer should be full
        assert!(buffer.is_full());
        assert_eq!(buffer.len(), 5);
        
        // Additional push should fail
        assert!(!buffer.push("overflow".to_string()));
        
        // Drain should return all items
        let items = buffer.drain();
        assert_eq!(items.len(), 5);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_media_event_types() {
        let audio_event = MediaEvent::AudioFrame {
            session_id: "s1".to_string(),
            timestamp_ms: 100,
            pcm_data: vec![0, 1, 2, 3],
            sample_rate: 16000,
            channels: 1,
        };
        
        if let MediaEvent::AudioFrame { sample_rate, .. } = audio_event {
            assert_eq!(sample_rate, 16000);
        } else {
            panic!("Wrong event type");
        }
    }

    #[tokio::test]
    async fn test_server_graceful_shutdown() {
        let config = Config {
            server: amwaj_media::config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 50098,
                worker_threads: 1,
            },
            ..Config::default()
        };
        let metrics = Arc::new(Metrics::new(&config));
        let server = GrpcServer::new(config, metrics);
        
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        
        let handle = tokio::spawn(async move {
            server.start_with_shutdown(shutdown_rx).await
        });
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let _ = shutdown_tx.send(());
        
        let result = handle.await;
        assert!(result.is_ok());
    }
}
