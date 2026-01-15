#[cfg(test)]
mod audio_tests {
    use amwaj_media::audio::{AudioProcessor, VoiceActivityDetector, VoiceIsolation};
    use amwaj_media::audio::features::{calculate_volume, calculate_zero_crossing_rate, extract_features};

    #[test]
    fn test_audio_processor_creation() {
        let processor = AudioProcessor::new(16000, 320);
        assert_eq!(processor.sample_rate(), 16000);
        assert_eq!(processor.frame_size(), 320);
    }

    #[test]
    fn test_vad_detection_silence() {
        let mut vad = VoiceActivityDetector::new(16000);
        let silent_audio = vec![0.0f32; 320];
        
        let prob = vad.process(&silent_audio).expect("VAD processing failed");
        assert!(prob < 0.5);
    }

    #[test]
    fn test_vad_detection_voice() {
        let mut vad = VoiceActivityDetector::new(16000);
        
        // Create "voice-like" signal (high energy)
        let voice_audio = vec![0.5f32; 320];
        
        let prob = vad.process(&voice_audio).expect("VAD processing failed");
        assert!(prob > 0.5);
    }

    #[test]
    fn test_volume_calculation() {
        let audio = vec![0.1f32; 320];
        let volume = calculate_volume(&audio);
        
        assert!(volume < 0.0); // dB of weak signal should be negative
    }

    #[test]
    fn test_volume_silence() {
        let audio = vec![0.0f32; 320];
        let volume = calculate_volume(&audio);
        
        assert!(volume.is_infinite() && volume.is_sign_negative());
    }

    #[test]
    fn test_zero_crossing_rate() {
        // High frequency alternating signal
        let audio: Vec<f32> = (0..320).map(|i| if i % 2 == 0 { 0.5 } else { -0.5 }).collect();
        let zcr = calculate_zero_crossing_rate(&audio);
        assert!(zcr > 0.9);
    }

    #[test]
    fn test_frame_processing_pipeline() {
        let mut processor = AudioProcessor::new(16000, 320);
        let pcm_data = vec![100i16; 320];
        
        let result = processor.process_frame(&pcm_data);
        assert!(result.is_ok());
        
        let frame = result.unwrap();
        assert!(frame.vad_probability >= 0.0 && frame.vad_probability <= 1.0);
        assert_eq!(frame.pcm.len(), 320);
    }

    #[test]
    fn test_audio_features_extraction() {
        let audio = vec![0.1f32; 320];
        let features = extract_features(&audio, 16000);
        
        assert!(features.volume_db < 0.0);
        assert!(features.zero_crossing_rate >= 0.0);
    }

    #[test]
    fn test_voice_isolation_stub() {
        let vi = VoiceIsolation::new("model.onnx".to_string()).unwrap();
        let audio = vec![0.5f32; 320];
        
        let result = vi.isolate(&audio);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 320);
    }

    #[test]
    fn test_multiple_frames() {
        let mut processor = AudioProcessor::new(16000, 320);
        
        for i in 0..10 {
            let pcm_data = vec![(i * 100) as i16; 320];
            let result = processor.process_frame(&pcm_data);
            assert!(result.is_ok());
        }
        
        assert_eq!(processor.frames_processed(), 10);
    }

    #[test]
    fn test_processor_reset() {
        let mut processor = AudioProcessor::new(16000, 320);
        
        processor.process_frame(&vec![100i16; 320]).unwrap();
        assert_eq!(processor.frames_processed(), 1);
        
        processor.reset();
        assert_eq!(processor.frames_processed(), 0);
    }
}
