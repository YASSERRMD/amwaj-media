//! Turn Detection Engine (stub for Phase 1)

#[allow(dead_code)]
pub struct TurnDetector {
    vad_sensitivity: f32,
    min_turn_duration_ms: u32,
    max_silence_duration_ms: u32,
}

impl TurnDetector {
    pub fn new(vad_sensitivity: f32, min_turn_duration_ms: u32, max_silence_duration_ms: u32) -> Self {
        Self {
            vad_sensitivity,
            min_turn_duration_ms,
            max_silence_duration_ms,
        }
    }

    pub fn vad_sensitivity(&self) -> f32 {
        self.vad_sensitivity
    }

    pub fn process_frame(&mut self, _pcm_data: &[i16]) -> Option<TurnEvent> {
        // TODO: Implement turn detection in Phase 4
        None
    }
}

#[derive(Debug, Clone)]
pub enum TurnEvent {
    TurnStarted { timestamp_ms: i64, vad_probability: f32 },
    TurnEnded { timestamp_ms: i64, duration_ms: u32 },
}
