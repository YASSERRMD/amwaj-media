//! Turn Detection Engine - State machine for voice turn-taking

use crate::audio::AudioFeatures;

/// State of the turn detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnState {
    /// No active speech
    Idle,
    /// User is currently speaking
    Speaking,
    /// Brief silence gap during/after speech
    SilenceGap,
}

/// Configuration for turn detection
#[derive(Debug, Clone)]
pub struct TurnDetectionConfig {
    /// VAD threshold to enter speaking state
    pub vad_threshold_enter: f32,
    /// VAD threshold to exit speaking state
    pub vad_threshold_exit: f32,
    /// Minimum speech duration to consider a valid turn (ms)
    pub min_speech_duration_ms: u32,
    /// Maximum silence duration before turn ends (ms)
    pub max_silence_duration_ms: u32,
    /// Minimum volume threshold (dB)
    pub volume_threshold_db: f32,
}

impl Default for TurnDetectionConfig {
    fn default() -> Self {
        Self {
            vad_threshold_enter: 0.6,
            vad_threshold_exit: 0.3,
            min_speech_duration_ms: 250,
            max_silence_duration_ms: 400,
            volume_threshold_db: -40.0,
        }
    }
}

/// Turn detection engine using state machine approach
pub struct TurnDetectionEngine {
    state: TurnState,
    vad_history: Vec<f32>,
    silence_duration_ms: u32,
    speech_duration_ms: u32,
    max_history_size: usize,
    config: TurnDetectionConfig,
    barge_in_pending: bool,
}

impl TurnDetectionEngine {
    /// Create a new turn detection engine
    pub fn new(config: TurnDetectionConfig) -> Self {
        Self {
            state: TurnState::Idle,
            vad_history: Vec::new(),
            silence_duration_ms: 0,
            speech_duration_ms: 0,
            max_history_size: 50,
            config,
            barge_in_pending: false,
        }
    }

    /// Process an audio frame and return any turn events
    pub fn process(
        &mut self,
        vad_prob: f32,
        features: &AudioFeatures,
        frame_duration_ms: u32,
    ) -> TurnEvent {
        // Update VAD history
        self.vad_history.push(vad_prob);
        if self.vad_history.len() > self.max_history_size {
            self.vad_history.remove(0);
        }

        match self.state {
            TurnState::Idle => self.handle_idle(vad_prob, features, frame_duration_ms),
            TurnState::Speaking => self.handle_speaking(vad_prob, features, frame_duration_ms),
            TurnState::SilenceGap => self.handle_silence_gap(vad_prob, features, frame_duration_ms),
        }
    }

    fn handle_idle(
        &mut self,
        vad_prob: f32,
        features: &AudioFeatures,
        frame_duration_ms: u32,
    ) -> TurnEvent {
        if vad_prob > self.config.vad_threshold_enter
            && features.volume_db > self.config.volume_threshold_db
        {
            self.state = TurnState::Speaking;
            self.speech_duration_ms = frame_duration_ms;
            TurnEvent::TurnStarted
        } else {
            TurnEvent::None
        }
    }

    fn handle_speaking(
        &mut self,
        vad_prob: f32,
        _features: &AudioFeatures,
        frame_duration_ms: u32,
    ) -> TurnEvent {
        self.speech_duration_ms += frame_duration_ms;

        if vad_prob < self.config.vad_threshold_exit {
            self.state = TurnState::SilenceGap;
            self.silence_duration_ms = frame_duration_ms;
            TurnEvent::None
        } else {
            TurnEvent::None
        }
    }

    fn handle_silence_gap(
        &mut self,
        vad_prob: f32,
        _features: &AudioFeatures,
        frame_duration_ms: u32,
    ) -> TurnEvent {
        self.silence_duration_ms += frame_duration_ms;

        if vad_prob > self.config.vad_threshold_enter {
            // Speech resumed, go back to speaking
            self.state = TurnState::Speaking;
            self.speech_duration_ms += frame_duration_ms;
            TurnEvent::None
        } else if self.silence_duration_ms >= self.config.max_silence_duration_ms {
            // Silence threshold exceeded, turn ended
            self.state = TurnState::Idle;
            let duration = self.speech_duration_ms;
            self.speech_duration_ms = 0;
            self.silence_duration_ms = 0;

            if duration >= self.config.min_speech_duration_ms {
                TurnEvent::TurnEnded(duration)
            } else {
                TurnEvent::None
            }
        } else {
            TurnEvent::None
        }
    }

    /// Get current state
    pub fn state(&self) -> TurnState {
        self.state
    }

    /// Get speech duration in ms
    pub fn speech_duration_ms(&self) -> u32 {
        self.speech_duration_ms
    }

    /// Get silence duration in ms
    pub fn silence_duration_ms(&self) -> u32 {
        self.silence_duration_ms
    }

    /// Reset the engine state
    pub fn reset(&mut self) {
        self.state = TurnState::Idle;
        self.vad_history.clear();
        self.silence_duration_ms = 0;
        self.speech_duration_ms = 0;
        self.barge_in_pending = false;
    }

    /// Get average VAD probability from history
    pub fn average_vad(&self) -> f32 {
        if self.vad_history.is_empty() {
            0.0
        } else {
            self.vad_history.iter().sum::<f32>() / self.vad_history.len() as f32
        }
    }

    /// Check if user is likely speaking
    pub fn is_speaking(&self) -> bool {
        self.state == TurnState::Speaking
    }

    /// Signal that a barge-in may have occurred
    pub fn signal_potential_barge_in(&mut self) {
        if self.state != TurnState::Idle {
            self.barge_in_pending = true;
        }
    }

    /// Check and consume barge-in flag
    pub fn check_barge_in(&mut self) -> bool {
        if self.barge_in_pending && self.state == TurnState::Speaking {
            self.barge_in_pending = false;
            true
        } else {
            false
        }
    }
}

/// Events emitted by the turn detection engine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurnEvent {
    /// No event
    None,
    /// User started speaking
    TurnStarted,
    /// User finished speaking (includes duration in ms)
    TurnEnded(u32),
    /// User interrupted (barge-in detected)
    BargeIn,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_features(volume_db: f32) -> AudioFeatures {
        AudioFeatures {
            volume_db,
            pitch_hz: 200.0,
            spectral_centroid: 0.0,
            zero_crossing_rate: 0.0,
        }
    }

    #[test]
    fn test_initial_state() {
        let engine = TurnDetectionEngine::new(TurnDetectionConfig::default());
        assert_eq!(engine.state(), TurnState::Idle);
    }

    #[test]
    fn test_idle_to_speaking() {
        let mut engine = TurnDetectionEngine::new(TurnDetectionConfig::default());
        let features = create_features(-20.0);
        
        let event = engine.process(0.8, &features, 20);
        assert_eq!(event, TurnEvent::TurnStarted);
        assert_eq!(engine.state(), TurnState::Speaking);
    }

    #[test]
    fn test_speaking_to_silence_gap() {
        let mut engine = TurnDetectionEngine::new(TurnDetectionConfig::default());
        let features = create_features(-20.0);
        
        // Start speaking
        engine.process(0.8, &features, 20);
        
        // Low VAD should transition to silence gap
        engine.process(0.1, &features, 20);
        assert_eq!(engine.state(), TurnState::SilenceGap);
    }

    #[test]
    fn test_turn_ended() {
        let config = TurnDetectionConfig {
            vad_threshold_enter: 0.6,
            vad_threshold_exit: 0.3,
            min_speech_duration_ms: 100,
            max_silence_duration_ms: 200,
            volume_threshold_db: -40.0,
        };
        
        let mut engine = TurnDetectionEngine::new(config);
        let features = create_features(-20.0);
        
        // Start speaking
        engine.process(0.8, &features, 20);
        
        // Continue speaking for enough time
        for _ in 0..10 {
            engine.process(0.8, &features, 20);
        }
        
        // Enter silence gap
        engine.process(0.1, &features, 20);
        
        // Wait for silence duration
        let mut turn_ended = false;
        for _ in 0..15 {
            if let TurnEvent::TurnEnded(_) = engine.process(0.1, &features, 20) {
                turn_ended = true;
                break;
            }
        }
        
        assert!(turn_ended);
        assert_eq!(engine.state(), TurnState::Idle);
    }

    #[test]
    fn test_speech_resume() {
        let mut engine = TurnDetectionEngine::new(TurnDetectionConfig::default());
        let features = create_features(-20.0);
        
        // Start speaking
        engine.process(0.8, &features, 20);
        
        // Brief silence
        engine.process(0.1, &features, 20);
        assert_eq!(engine.state(), TurnState::SilenceGap);
        
        // Resume speaking
        engine.process(0.8, &features, 20);
        assert_eq!(engine.state(), TurnState::Speaking);
    }

    #[test]
    fn test_reset() {
        let mut engine = TurnDetectionEngine::new(TurnDetectionConfig::default());
        let features = create_features(-20.0);
        
        engine.process(0.8, &features, 20);
        assert_eq!(engine.state(), TurnState::Speaking);
        
        engine.reset();
        assert_eq!(engine.state(), TurnState::Idle);
        assert_eq!(engine.speech_duration_ms(), 0);
    }
}
