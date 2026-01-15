//! Audio Feature Extraction

/// Audio features extracted from a frame
#[derive(Debug, Clone, Default)]
pub struct AudioFeatures {
    /// Volume in decibels (dB)
    pub volume_db: f32,
    /// Estimated pitch in Hz
    pub pitch_hz: f32,
    /// Spectral centroid
    pub spectral_centroid: f32,
    /// Zero crossing rate
    pub zero_crossing_rate: f32,
}

impl AudioFeatures {
    /// Create a new AudioFeatures instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the audio is likely speech based on features
    pub fn is_likely_speech(&self) -> bool {
        // Human speech typically falls between 85-255 Hz for fundamental frequency
        self.pitch_hz > 50.0 && self.pitch_hz < 400.0 && self.volume_db > -50.0
    }
}

/// Calculate volume (RMS) in decibels
pub fn calculate_volume(audio: &[f32]) -> f32 {
    if audio.is_empty() {
        return f32::NEG_INFINITY;
    }

    let mean_square: f32 = audio.iter().map(|x| x * x).sum::<f32>() / audio.len() as f32;
    let rms = mean_square.sqrt();

    if rms > 0.0 {
        20.0 * rms.log10()
    } else {
        f32::NEG_INFINITY
    }
}

/// Estimate fundamental frequency (pitch) using autocorrelation
pub fn estimate_pitch(audio: &[f32], sample_rate: u32) -> f32 {
    if audio.len() < 100 {
        return 0.0;
    }

    // Simple autocorrelation-based pitch detection
    let min_period = (sample_rate / 400) as usize; // Max 400 Hz
    let max_period = (sample_rate / 50) as usize; // Min 50 Hz

    if max_period >= audio.len() || min_period >= max_period {
        return 0.0;
    }

    let mut best_correlation = 0.0f32;
    let mut best_period = 0;

    for period in min_period..max_period.min(audio.len() / 2) {
        let mut correlation = 0.0f32;
        let mut norm1 = 0.0f32;
        let mut norm2 = 0.0f32;

        for i in 0..(audio.len() - period) {
            correlation += audio[i] * audio[i + period];
            norm1 += audio[i] * audio[i];
            norm2 += audio[i + period] * audio[i + period];
        }

        let normalized = if norm1 > 0.0 && norm2 > 0.0 {
            correlation / (norm1.sqrt() * norm2.sqrt())
        } else {
            0.0
        };

        if normalized > best_correlation {
            best_correlation = normalized;
            best_period = period;
        }
    }

    if best_period > 0 && best_correlation > 0.6 {
        sample_rate as f32 / best_period as f32
    } else {
        0.0
    }
}

/// Calculate zero crossing rate
pub fn calculate_zero_crossing_rate(audio: &[f32]) -> f32 {
    if audio.len() < 2 {
        return 0.0;
    }

    let crossings: usize = audio
        .windows(2)
        .filter(|w| (w[0] >= 0.0 && w[1] < 0.0) || (w[0] < 0.0 && w[1] >= 0.0))
        .count();

    crossings as f32 / (audio.len() - 1) as f32
}

/// Calculate spectral centroid (simplified version without FFT)
pub fn calculate_spectral_centroid(audio: &[f32], sample_rate: u32) -> f32 {
    if audio.is_empty() {
        return 0.0;
    }

    // Simplified: use zero crossing rate as a proxy for spectral centroid
    // A proper implementation would use FFT
    let zcr = calculate_zero_crossing_rate(audio);
    zcr * sample_rate as f32 / 2.0
}

/// Extract all audio features from a frame
pub fn extract_features(audio: &[f32], sample_rate: u32) -> AudioFeatures {
    AudioFeatures {
        volume_db: calculate_volume(audio),
        pitch_hz: estimate_pitch(audio, sample_rate),
        spectral_centroid: calculate_spectral_centroid(audio, sample_rate),
        zero_crossing_rate: calculate_zero_crossing_rate(audio),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_volume_silence() {
        let audio = vec![0.0f32; 320];
        let vol = calculate_volume(&audio);
        assert!(vol == f32::NEG_INFINITY);
    }

    #[test]
    fn test_calculate_volume_signal() {
        let audio = vec![0.1f32; 320];
        let vol = calculate_volume(&audio);
        assert!(vol < 0.0); // dB of 0.1 RMS should be negative
        assert!(vol > -30.0); // But not too negative
    }

    #[test]
    fn test_zero_crossing_rate() {
        // Alternating signal has high ZCR
        let audio: Vec<f32> = (0..100)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();
        let zcr = calculate_zero_crossing_rate(&audio);
        assert!(zcr > 0.9);
    }

    #[test]
    fn test_features_default() {
        let features = AudioFeatures::default();
        assert_eq!(features.volume_db, 0.0);
        assert_eq!(features.pitch_hz, 0.0);
    }
}
