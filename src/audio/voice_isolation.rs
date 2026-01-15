//! Enhanced Voice Isolation with ONNX Runtime support
//!
//! This module provides voice isolation using ONNX models.
//! When the `audio-feature` is enabled, it uses ort for inference.

use std::path::Path;

/// Voice isolation configuration
#[derive(Debug, Clone)]
pub struct VoiceIsolationConfig {
    /// Path to the ONNX model
    pub model_path: String,
    /// Sample rate (default: 16000)
    pub sample_rate: u32,
    /// Use GPU for inference
    pub use_gpu: bool,
    /// GPU device ID
    pub gpu_device_id: i32,
    /// Batch size for inference
    pub batch_size: usize,
}

impl Default for VoiceIsolationConfig {
    fn default() -> Self {
        Self {
            model_path: "models/voice_isolation.onnx".to_string(),
            sample_rate: 16000,
            use_gpu: false,
            gpu_device_id: 0,
            batch_size: 1,
        }
    }
}

/// Voice isolation processor using ONNX model
#[allow(dead_code)]
pub struct VoiceIsolation {
    config: VoiceIsolationConfig,
    enabled: bool,
    frames_processed: u64,
}

impl VoiceIsolation {
    /// Create a new voice isolation processor
    pub fn new(model_path: String) -> anyhow::Result<Self> {
        let config = VoiceIsolationConfig {
            model_path,
            ..VoiceIsolationConfig::default()
        };
        Self::with_config(config)
    }

    /// Create with full configuration
    pub fn with_config(config: VoiceIsolationConfig) -> anyhow::Result<Self> {
        // TODO: Load ONNX model when audio-feature is enabled
        // For now, just validate the model path exists or use stub
        if !config.model_path.is_empty() && Path::new(&config.model_path).exists() {
            tracing::info!("Voice isolation model found at: {}", config.model_path);
        } else {
            tracing::debug!(
                "Voice isolation model not found, using stub: {}",
                config.model_path
            );
        }

        Ok(Self {
            config,
            enabled: true,
            frames_processed: 0,
        })
    }

    /// Create with custom sample rate
    pub fn with_sample_rate(model_path: String, sample_rate: u32) -> anyhow::Result<Self> {
        let config = VoiceIsolationConfig {
            model_path,
            sample_rate,
            ..VoiceIsolationConfig::default()
        };
        Self::with_config(config)
    }

    /// Download model from Hugging Face Hub (stub)
    pub async fn from_hub(
        _repo_id: &str,
        _filename: &str,
        sample_rate: u32,
        _cache_dir: Option<&str>,
    ) -> anyhow::Result<Self> {
        // TODO: Implement actual Hugging Face Hub download
        // For now, create a stub processor
        let config = VoiceIsolationConfig {
            model_path: "models/voice_isolation.onnx".to_string(),
            sample_rate,
            ..VoiceIsolationConfig::default()
        };
        Self::with_config(config)
    }

    /// Isolate voice from audio signal
    ///
    /// When ONNX is available, runs inference to separate voice from noise.
    /// Otherwise, applies a simple noise gate.
    pub fn isolate(&mut self, audio: &[f32]) -> anyhow::Result<Vec<f32>> {
        if !self.enabled {
            return Ok(audio.to_vec());
        }

        self.frames_processed += 1;

        // TODO: When `audio-feature` is enabled, use ONNX inference:
        // let input = Array2::from_shape_vec((1, audio.len()), audio.to_vec())?;
        // let outputs = self.session.run(inputs![input])?;
        // let output = outputs[0].try_extract_tensor::<f32>()?;

        // For now, apply simple noise gate as placeholder
        let threshold = 0.02;
        let ratio = 0.1; // Reduction ratio for below-threshold samples

        let output: Vec<f32> = audio
            .iter()
            .map(|&s| if s.abs() > threshold { s } else { s * ratio })
            .collect();

        Ok(output)
    }

    /// Process i16 PCM audio
    pub fn isolate_i16(&mut self, audio: &[i16]) -> anyhow::Result<Vec<i16>> {
        let float_audio: Vec<f32> = audio.iter().map(|&s| s as f32 / 32768.0).collect();

        let processed = self.isolate(&float_audio)?;

        Ok(processed
            .iter()
            .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect())
    }

    /// Enable or disable voice isolation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if voice isolation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the model path
    pub fn model_path(&self) -> &str {
        &self.config.model_path
    }

    /// Get frames processed count
    pub fn frames_processed(&self) -> u64 {
        self.frames_processed
    }

    /// Get configuration
    pub fn config(&self) -> &VoiceIsolationConfig {
        &self.config
    }

    /// Reset processor state
    pub fn reset(&mut self) {
        self.frames_processed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_isolation_creation() {
        let vi = VoiceIsolation::new("model.onnx".to_string());
        assert!(vi.is_ok());

        let vi = vi.unwrap();
        assert!(vi.is_enabled());
        assert_eq!(vi.model_path(), "model.onnx");
    }

    #[test]
    fn test_config_defaults() {
        let config = VoiceIsolationConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert!(!config.use_gpu);
        assert_eq!(config.batch_size, 1);
    }

    #[test]
    fn test_isolate_passthrough() {
        let mut vi = VoiceIsolation::new("model.onnx".to_string()).unwrap();
        let audio = vec![0.5f32; 320];

        let result = vi.isolate(&audio);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.len(), 320);
        assert_eq!(vi.frames_processed(), 1);
    }

    #[test]
    fn test_disable_isolation() {
        let mut vi = VoiceIsolation::new("model.onnx".to_string()).unwrap();
        let audio = vec![0.5f32; 320];

        vi.set_enabled(false);
        assert!(!vi.is_enabled());

        let result = vi.isolate(&audio).unwrap();
        assert_eq!(result, audio); // Should pass through unchanged
    }

    #[test]
    fn test_i16_processing() {
        let mut vi = VoiceIsolation::new("model.onnx".to_string()).unwrap();
        let audio = vec![16000i16; 320];

        let result = vi.isolate_i16(&audio);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 320);
    }

    #[test]
    fn test_reset() {
        let mut vi = VoiceIsolation::new("model.onnx".to_string()).unwrap();

        vi.isolate(&vec![0.1f32; 320]).unwrap();
        vi.isolate(&vec![0.1f32; 320]).unwrap();
        assert_eq!(vi.frames_processed(), 2);

        vi.reset();
        assert_eq!(vi.frames_processed(), 0);
    }

    #[tokio::test]
    async fn test_from_hub_stub() {
        let vi = VoiceIsolation::from_hub("repo/model", "model.onnx", 16000, None).await;
        assert!(vi.is_ok());
    }
}
