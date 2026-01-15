//! Audio Processor - Main audio processing pipeline

use crate::audio::features::extract_features;
use crate::audio::{AudioFeatures, VoiceActivityDetector, VoiceIsolation};

/// Main audio processor that orchestrates the audio pipeline
pub struct AudioProcessor {
    sample_rate: u32,
    frame_size: usize,
    voice_isolation: Option<VoiceIsolation>,
    vad: VoiceActivityDetector,
    frames_processed: u64,
}

/// Result of processing an audio frame
#[derive(Debug, Clone)]
pub struct ProcessedFrame {
    /// Processed audio samples
    pub pcm: Vec<f32>,
    /// Extracted audio features
    pub features: AudioFeatures,
    /// Voice activity probability (0.0 - 1.0)
    pub vad_probability: f32,
    /// Frame timestamp
    pub timestamp_ms: i64,
}

impl AudioProcessor {
    /// Create a new audio processor
    pub fn new(sample_rate: u32, frame_size: usize) -> Self {
        Self {
            sample_rate,
            frame_size,
            voice_isolation: None,
            vad: VoiceActivityDetector::new(sample_rate),
            frames_processed: 0,
        }
    }

    /// Create with voice isolation enabled
    pub fn with_voice_isolation(
        sample_rate: u32,
        frame_size: usize,
        model_path: String,
    ) -> anyhow::Result<Self> {
        let vi = VoiceIsolation::new(model_path)?;
        Ok(Self {
            sample_rate,
            frame_size,
            voice_isolation: Some(vi),
            vad: VoiceActivityDetector::new(sample_rate),
            frames_processed: 0,
        })
    }

    /// Process an audio frame (PCM i16)
    pub fn process_frame(&mut self, pcm_data: &[i16]) -> anyhow::Result<ProcessedFrame> {
        self.frames_processed += 1;

        // Convert to float
        let float_data = pcm_to_float(pcm_data);

        // Apply voice isolation if available
        let isolated = if let Some(vi) = &self.voice_isolation {
            vi.isolate(&float_data)?
        } else {
            float_data
        };

        // Extract audio features
        let features = extract_features(&isolated, self.sample_rate);

        // Run VAD
        let vad_prob = self.vad.process(&isolated)?;

        // Calculate timestamp
        let timestamp_ms = self.calculate_timestamp();

        Ok(ProcessedFrame {
            pcm: isolated,
            features,
            vad_probability: vad_prob,
            timestamp_ms,
        })
    }

    /// Process float audio frame directly
    pub fn process_frame_float(&mut self, float_data: &[f32]) -> anyhow::Result<ProcessedFrame> {
        self.frames_processed += 1;

        // Apply voice isolation if available
        let isolated = if let Some(vi) = &self.voice_isolation {
            vi.isolate(float_data)?
        } else {
            float_data.to_vec()
        };

        // Extract audio features
        let features = extract_features(&isolated, self.sample_rate);

        // Run VAD
        let vad_prob = self.vad.process(&isolated)?;

        // Calculate timestamp
        let timestamp_ms = self.calculate_timestamp();

        Ok(ProcessedFrame {
            pcm: isolated,
            features,
            vad_probability: vad_prob,
            timestamp_ms,
        })
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get frame size
    pub fn frame_size(&self) -> usize {
        self.frame_size
    }

    /// Get frames processed count
    pub fn frames_processed(&self) -> u64 {
        self.frames_processed
    }

    /// Reset processor state
    pub fn reset(&mut self) {
        self.vad.reset();
        self.frames_processed = 0;
    }

    /// Enable or disable voice isolation
    pub fn set_voice_isolation_enabled(&mut self, enabled: bool) {
        if let Some(vi) = &mut self.voice_isolation {
            vi.set_enabled(enabled);
        }
    }

    fn calculate_timestamp(&self) -> i64 {
        let frame_duration_ms = (self.frame_size as f64 / self.sample_rate as f64) * 1000.0;
        (self.frames_processed as f64 * frame_duration_ms) as i64
    }
}

/// Convert PCM i16 samples to float
pub fn pcm_to_float(pcm: &[i16]) -> Vec<f32> {
    pcm.iter().map(|&x| x as f32 / 32768.0).collect()
}

/// Convert float samples to PCM i16
pub fn float_to_pcm(float_data: &[f32]) -> Vec<i16> {
    float_data
        .iter()
        .map(|&x| (x * 32767.0).clamp(-32768.0, 32767.0) as i16)
        .collect()
}

/// Re-export calculate_volume for tests
pub use crate::audio::features::calculate_volume as calc_volume;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = AudioProcessor::new(16000, 320);
        assert_eq!(processor.sample_rate(), 16000);
        assert_eq!(processor.frame_size(), 320);
        assert_eq!(processor.frames_processed(), 0);
    }

    #[test]
    fn test_process_frame() {
        let mut processor = AudioProcessor::new(16000, 320);
        let pcm_data = vec![100i16; 320];

        let result = processor.process_frame(&pcm_data);
        assert!(result.is_ok());

        let frame = result.unwrap();
        assert_eq!(frame.pcm.len(), 320);
        assert!(frame.vad_probability >= 0.0 && frame.vad_probability <= 1.0);
        assert_eq!(processor.frames_processed(), 1);
    }

    #[test]
    fn test_process_silence() {
        let mut processor = AudioProcessor::new(16000, 320);
        let silent_data = vec![0i16; 320];

        let frame = processor.process_frame(&silent_data).unwrap();
        assert!(frame.vad_probability < 0.5);
    }

    #[test]
    fn test_pcm_conversion_roundtrip() {
        let original = vec![100i16, -200, 32000, -32000, 0];
        let float_data = pcm_to_float(&original);
        let back = float_to_pcm(&float_data);

        for (o, b) in original.iter().zip(back.iter()) {
            assert!((o - b).abs() <= 1); // Allow for rounding
        }
    }

    #[test]
    fn test_volume_calculation() {
        let audio = vec![0.1f32; 320];
        let vol = calc_volume(&audio);
        assert!(vol < 0.0 && vol > -30.0);
    }
}
