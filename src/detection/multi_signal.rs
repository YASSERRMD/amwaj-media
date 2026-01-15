//! Multi-Signal Fusion for turn detection

use crate::audio::AudioFeatures;

/// Multi-signal fusion combines VAD, volume, pitch, and context signals
pub struct MultiSignalFusion {
    vad_weight: f32,
    volume_weight: f32,
    pitch_weight: f32,
    context_weight: f32,
}

impl MultiSignalFusion {
    /// Create a new multi-signal fusion processor with default weights
    pub fn new() -> Self {
        Self {
            vad_weight: 0.5,
            volume_weight: 0.3,
            pitch_weight: 0.1,
            context_weight: 0.1,
        }
    }

    /// Create with custom weights
    pub fn with_weights(vad: f32, volume: f32, pitch: f32, context: f32) -> Self {
        Self {
            vad_weight: vad,
            volume_weight: volume,
            pitch_weight: pitch,
            context_weight: context,
        }
    }

    /// Fuse multiple signals into a single confidence score
    pub fn fuse_signals(
        &self,
        vad_prob: f32,
        features: &AudioFeatures,
        context: Option<&str>,
    ) -> f32 {
        // Normalize volume: map -50db to 0db range to 0-1
        let volume_normalized = ((features.volume_db + 50.0) / 50.0).clamp(0.0, 1.0);

        // Pitch score: human speech typically 50-400 Hz
        let pitch_score = if features.pitch_hz > 50.0 && features.pitch_hz < 400.0 {
            1.0
        } else if features.pitch_hz > 0.0 {
            0.3 // Some pitch detected but outside normal range
        } else {
            0.0
        };

        // Context boost based on conversation state
        let context_boost = match context {
            Some("expecting_response") => 0.2,
            Some("user_speaking") => 0.1,
            Some("thinking") => -0.1,
            Some("playing_audio") => -0.2,
            _ => 0.0,
        };

        // Weighted combination
        let base_score = vad_prob * self.vad_weight
            + volume_normalized * self.volume_weight
            + pitch_score * self.pitch_weight;

        // Apply context adjustment
        let fused = base_score + context_boost * self.context_weight;

        fused.clamp(0.0, 1.0)
    }

    /// Get a confidence level classification
    pub fn confidence_level(&self, score: f32) -> ConfidenceLevel {
        if score >= 0.8 {
            ConfidenceLevel::High
        } else if score >= 0.5 {
            ConfidenceLevel::Medium
        } else if score >= 0.2 {
            ConfidenceLevel::Low
        } else {
            ConfidenceLevel::VeryLow
        }
    }

    /// Update weights dynamically
    pub fn set_weights(&mut self, vad: f32, volume: f32, pitch: f32, context: f32) {
        self.vad_weight = vad;
        self.volume_weight = volume;
        self.pitch_weight = pitch;
        self.context_weight = context;
    }
}

impl Default for MultiSignalFusion {
    fn default() -> Self {
        Self::new()
    }
}

/// Confidence level for fused signal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    VeryLow,
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_features(volume_db: f32, pitch_hz: f32) -> AudioFeatures {
        AudioFeatures {
            volume_db,
            pitch_hz,
            spectral_centroid: 0.0,
            zero_crossing_rate: 0.0,
        }
    }

    #[test]
    fn test_fusion_high_confidence() {
        let fusion = MultiSignalFusion::new();
        let features = create_features(-20.0, 200.0);

        let score = fusion.fuse_signals(0.9, &features, None);
        assert!(score > 0.7);
    }

    #[test]
    fn test_fusion_low_confidence() {
        let fusion = MultiSignalFusion::new();
        let features = create_features(-60.0, 0.0);

        let score = fusion.fuse_signals(0.1, &features, None);
        assert!(score < 0.3);
    }

    #[test]
    fn test_context_boost() {
        let fusion = MultiSignalFusion::new();
        let features = create_features(-30.0, 150.0);

        let score_neutral = fusion.fuse_signals(0.5, &features, None);
        let score_expecting = fusion.fuse_signals(0.5, &features, Some("expecting_response"));
        let score_playing = fusion.fuse_signals(0.5, &features, Some("playing_audio"));

        assert!(score_expecting > score_neutral);
        assert!(score_playing < score_neutral);
    }

    #[test]
    fn test_confidence_levels() {
        let fusion = MultiSignalFusion::new();

        assert_eq!(fusion.confidence_level(0.9), ConfidenceLevel::High);
        assert_eq!(fusion.confidence_level(0.6), ConfidenceLevel::Medium);
        assert_eq!(fusion.confidence_level(0.3), ConfidenceLevel::Low);
        assert_eq!(fusion.confidence_level(0.1), ConfidenceLevel::VeryLow);
    }

    #[test]
    fn test_custom_weights() {
        let fusion = MultiSignalFusion::with_weights(0.8, 0.1, 0.05, 0.05);
        let features = create_features(-20.0, 200.0);

        // With higher VAD weight, high VAD should dominate
        let score = fusion.fuse_signals(0.9, &features, None);
        assert!(score > 0.7);
    }

    #[test]
    fn test_clamping() {
        let fusion = MultiSignalFusion::new();
        let features = create_features(10.0, 300.0); // Very loud

        let score = fusion.fuse_signals(1.0, &features, Some("expecting_response"));

        // Should be clamped to 1.0
        assert!(score <= 1.0);
        assert!(score >= 0.0);
    }
}
