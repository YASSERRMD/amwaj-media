//! Voice Activity Detection (VAD)

/// Voice Activity Detector using energy-based detection
pub struct VoiceActivityDetector {
    sample_rate: u32,
    energy_threshold: f32,
    smoothing_factor: f32,
    previous_prob: f32,
    frame_count: u64,
}

impl VoiceActivityDetector {
    /// Create a new VAD instance
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            energy_threshold: 0.001,
            smoothing_factor: 0.7,
            previous_prob: 0.0,
            frame_count: 0,
        }
    }

    /// Create VAD with custom threshold
    pub fn with_threshold(sample_rate: u32, threshold: f32) -> Self {
        Self {
            sample_rate,
            energy_threshold: threshold,
            smoothing_factor: 0.7,
            previous_prob: 0.0,
            frame_count: 0,
        }
    }

    /// Process an audio frame and return VAD probability
    pub fn process(&mut self, audio: &[f32]) -> anyhow::Result<f32> {
        if audio.is_empty() {
            return Ok(0.0);
        }

        self.frame_count += 1;

        // Calculate frame energy
        let energy = audio.iter().map(|x| x * x).sum::<f32>() / audio.len() as f32;

        // Calculate raw probability based on energy
        let raw_prob = if energy > self.energy_threshold {
            // Logarithmic scaling for better sensitivity
            let ratio = (energy / self.energy_threshold).ln();
            (ratio / 5.0).clamp(0.0, 1.0) // Scale and clamp
        } else {
            0.0
        };

        // Apply temporal smoothing
        let smoothed_prob =
            self.smoothing_factor * raw_prob + (1.0 - self.smoothing_factor) * self.previous_prob;

        self.previous_prob = smoothed_prob;

        Ok(smoothed_prob)
    }

    /// Process PCM i16 audio frame
    pub fn process_i16(&mut self, audio: &[i16]) -> anyhow::Result<f32> {
        let float_audio: Vec<f32> = audio.iter().map(|&s| s as f32 / 32768.0).collect();
        self.process(&float_audio)
    }

    /// Reset the VAD state
    pub fn reset(&mut self) {
        self.previous_prob = 0.0;
        self.frame_count = 0;
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the number of frames processed
    pub fn frames_processed(&self) -> u64 {
        self.frame_count
    }

    /// Update the energy threshold adaptively
    pub fn adapt_threshold(&mut self, noise_floor: f32) {
        // Set threshold slightly above noise floor
        self.energy_threshold = noise_floor * 2.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_creation() {
        let vad = VoiceActivityDetector::new(16000);
        assert_eq!(vad.sample_rate(), 16000);
        assert_eq!(vad.frames_processed(), 0);
    }

    #[test]
    fn test_vad_silence() {
        let mut vad = VoiceActivityDetector::new(16000);
        let silent_audio = vec![0.0f32; 320];

        let prob = vad.process(&silent_audio).unwrap();
        assert!(prob < 0.1);
    }

    #[test]
    fn test_vad_voice() {
        let mut vad = VoiceActivityDetector::new(16000);

        // Create high-energy "voice" signal
        let voice_audio = vec![0.5f32; 320];

        let prob = vad.process(&voice_audio).unwrap();
        assert!(prob > 0.5);
    }

    #[test]
    fn test_vad_smoothing() {
        let mut vad = VoiceActivityDetector::new(16000);

        // Process silence then voice, check smoothing
        let _prob1 = vad.process(&vec![0.0f32; 320]).unwrap();
        let prob2 = vad.process(&vec![0.5f32; 320]).unwrap();

        // Due to smoothing, prob2 shouldn't immediately jump to max
        assert!(prob2 > 0.0 && prob2 < 1.0);
    }

    #[test]
    fn test_vad_reset() {
        let mut vad = VoiceActivityDetector::new(16000);

        vad.process(&vec![0.5f32; 320]).unwrap();
        assert!(vad.frames_processed() > 0);

        vad.reset();
        assert_eq!(vad.frames_processed(), 0);
    }
}
