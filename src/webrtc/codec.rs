//! Opus Codec Handler
//!
//! Provides Opus encoding/decoding for WebRTC audio streams.
//! When the `opus-feature` is enabled, uses the audiopus crate.

/// Opus codec configuration
#[derive(Debug, Clone)]
pub struct OpusConfig {
    /// Sample rate (8000, 12000, 16000, 24000, 48000)
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u8,
    /// Target bitrate in bits per second
    pub bitrate: u32,
    /// Encoder complexity (0-10, higher = better quality but slower)
    pub complexity: u8,
    /// Enable discontinuous transmission
    pub use_dtx: bool,
    /// Enable forward error correction
    pub use_fec: bool,
    /// Frame size in samples (120, 240, 480, 960, 1920, 2880)
    pub frame_size: usize,
}

impl Default for OpusConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bitrate: 28000, // 28 kbps - optimal for voice
            complexity: 9,  // High quality
            use_dtx: true,
            use_fec: true,
            frame_size: 320, // 20ms at 16kHz
        }
    }
}

/// Opus decoder
pub struct OpusDecoder {
    sample_rate: u32,
    channels: u8,
    frames_decoded: u64,
}

impl OpusDecoder {
    /// Create a new Opus decoder
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            channels: 1,
            frames_decoded: 0,
        }
    }

    /// Create decoder with configuration
    pub fn with_config(config: &OpusConfig) -> anyhow::Result<Self> {
        // TODO: When opus-feature is enabled:
        // let decoder = audiopus::coder::Decoder::new(
        //     audiopus::SampleRate::Hz16000,
        //     audiopus::Channels::Mono,
        // )?;
        Ok(Self {
            sample_rate: config.sample_rate,
            channels: config.channels,
            frames_decoded: 0,
        })
    }

    /// Decode Opus data to PCM
    pub fn decode(&mut self, opus_data: &[u8]) -> anyhow::Result<Vec<i16>> {
        if opus_data.is_empty() {
            return Err(anyhow::anyhow!("Empty opus data"));
        }

        self.frames_decoded += 1;

        // TODO: When opus-feature is enabled:
        // let mut pcm = vec![0i16; self.frame_size * self.channels as usize];
        // let decoded_samples = self.decoder.decode(
        //     Some(opus_data),
        //     &mut pcm,
        //     false
        // )?;

        // Stub: Generate silence proportional to input
        // Real Opus decoding would produce actual audio
        let samples_per_frame = (self.sample_rate / 50) as usize; // 20ms frame
        let pcm = vec![0i16; samples_per_frame * self.channels as usize];

        Ok(pcm)
    }

    /// Decode with FEC (forward error correction)
    pub fn decode_fec(&mut self, opus_data: Option<&[u8]>) -> anyhow::Result<Vec<i16>> {
        match opus_data {
            Some(data) => self.decode(data),
            None => {
                // Generate PLC (packet loss concealment) frame
                let samples_per_frame = (self.sample_rate / 50) as usize;
                Ok(vec![0i16; samples_per_frame * self.channels as usize])
            }
        }
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get frames decoded count
    pub fn frames_decoded(&self) -> u64 {
        self.frames_decoded
    }

    /// Reset decoder state
    pub fn reset(&mut self) {
        self.frames_decoded = 0;
    }
}

/// Opus encoder
pub struct OpusEncoder {
    config: OpusConfig,
    frames_encoded: u64,
    adaptive_bitrate_enabled: bool,
    current_bitrate: u32,
}

impl OpusEncoder {
    /// Create a new Opus encoder
    pub fn new(sample_rate: u32) -> Self {
        let config = OpusConfig {
            sample_rate,
            ..OpusConfig::default()
        };
        Self {
            current_bitrate: config.bitrate,
            config,
            frames_encoded: 0,
            adaptive_bitrate_enabled: false,
        }
    }

    /// Create encoder with configuration
    pub fn with_config(config: OpusConfig) -> anyhow::Result<Self> {
        // TODO: When opus-feature is enabled:
        // let encoder = audiopus::coder::Encoder::new(
        //     audiopus::SampleRate::Hz16000,
        //     audiopus::Channels::Mono,
        //     audiopus::Application::Voip,
        // )?;
        // encoder.set_bitrate(audiopus::Bitrate::BitsPerSecond(config.bitrate as i32))?;
        Ok(Self {
            current_bitrate: config.bitrate,
            config,
            frames_encoded: 0,
            adaptive_bitrate_enabled: false,
        })
    }

    /// Encode PCM to Opus
    pub fn encode(&mut self, pcm_data: &[i16]) -> anyhow::Result<Vec<u8>> {
        if pcm_data.is_empty() {
            return Err(anyhow::anyhow!("Empty PCM data"));
        }

        self.frames_encoded += 1;

        // TODO: When opus-feature is enabled:
        // let mut opus_data = vec![0u8; 1500]; // Max packet size
        // let encoded_size = self.encoder.encode(pcm_data, &mut opus_data)?;
        // opus_data.truncate(encoded_size);

        // Stub: Return fake opus data
        // Size based on bitrate approximation
        let bytes_per_frame = (self.current_bitrate / 8 / 50) as usize; // 20ms frame
        let opus_data = vec![0xFFu8; bytes_per_frame.max(10)];

        Ok(opus_data)
    }

    /// Enable adaptive bitrate
    pub fn enable_adaptive_bitrate(&mut self) {
        self.adaptive_bitrate_enabled = true;
    }

    /// Disable adaptive bitrate
    pub fn disable_adaptive_bitrate(&mut self) {
        self.adaptive_bitrate_enabled = false;
        self.current_bitrate = self.config.bitrate;
    }

    /// Adapt bitrate based on network conditions
    pub fn adapt_bitrate(&mut self, packet_loss_percent: f32, available_bandwidth_kbps: u32) {
        if !self.adaptive_bitrate_enabled {
            return;
        }

        // Reduce bitrate with high packet loss
        let loss_factor = if packet_loss_percent > 10.0 {
            0.7
        } else if packet_loss_percent > 5.0 {
            0.85
        } else {
            1.0
        };

        // Cap at available bandwidth
        let max_bitrate = (available_bandwidth_kbps * 1000) as f32 * 0.8; // 80% of available

        let target = (self.config.bitrate as f32 * loss_factor).min(max_bitrate) as u32;

        // Clamp to valid range (6kbps - 510kbps for Opus)
        self.current_bitrate = target.clamp(6000, 510000);
    }

    /// Get current bitrate
    pub fn current_bitrate(&self) -> u32 {
        self.current_bitrate
    }

    /// Get frames encoded count
    pub fn frames_encoded(&self) -> u64 {
        self.frames_encoded
    }

    /// Get configuration
    pub fn config(&self) -> &OpusConfig {
        &self.config
    }

    /// Reset encoder state
    pub fn reset(&mut self) {
        self.frames_encoded = 0;
        self.current_bitrate = self.config.bitrate;
    }
}

/// Combined codec manager for encoding and decoding
pub struct OpusCodecManager {
    encoder: OpusEncoder,
    decoder: OpusDecoder,
}

impl OpusCodecManager {
    /// Create a new codec manager
    pub fn new(config: OpusConfig) -> anyhow::Result<Self> {
        let encoder = OpusEncoder::with_config(config.clone())?;
        let decoder = OpusDecoder::with_config(&config)?;
        Ok(Self { encoder, decoder })
    }

    /// Encode PCM to Opus
    pub fn encode(&mut self, pcm_data: &[i16]) -> anyhow::Result<Vec<u8>> {
        self.encoder.encode(pcm_data)
    }

    /// Decode Opus to PCM
    pub fn decode(&mut self, opus_data: &[u8]) -> anyhow::Result<Vec<i16>> {
        self.decoder.decode(opus_data)
    }

    /// Enable adaptive bitrate
    pub fn enable_adaptive_bitrate(&mut self) {
        self.encoder.enable_adaptive_bitrate();
    }

    /// Adapt bitrate
    pub fn adapt_bitrate(&mut self, packet_loss_percent: f32, available_bandwidth_kbps: u32) {
        self.encoder
            .adapt_bitrate(packet_loss_percent, available_bandwidth_kbps);
    }

    /// Get encoder reference
    pub fn encoder(&self) -> &OpusEncoder {
        &self.encoder
    }

    /// Get decoder reference
    pub fn decoder(&self) -> &OpusDecoder {
        &self.decoder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_creation() {
        let decoder = OpusDecoder::new(16000);
        assert_eq!(decoder.sample_rate(), 16000);
        assert_eq!(decoder.frames_decoded(), 0);
    }

    #[test]
    fn test_decode_empty_fails() {
        let mut decoder = OpusDecoder::new(16000);
        let result = decoder.decode(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_stub() {
        let mut decoder = OpusDecoder::new(16000);
        let opus_data = vec![0xFF, 0x00, 0xAB];

        let result = decoder.decode(&opus_data);
        assert!(result.is_ok());

        let pcm = result.unwrap();
        assert!(!pcm.is_empty());
        assert_eq!(decoder.frames_decoded(), 1);
    }

    #[test]
    fn test_encoder_creation() {
        let encoder = OpusEncoder::new(16000);
        assert_eq!(encoder.config().sample_rate, 16000);
        assert_eq!(encoder.frames_encoded(), 0);
    }

    #[test]
    fn test_encode_stub() {
        let mut encoder = OpusEncoder::new(16000);
        let pcm = vec![100i16; 320];

        let result = encoder.encode(&pcm);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
        assert_eq!(encoder.frames_encoded(), 1);
    }

    #[test]
    fn test_adaptive_bitrate() {
        let mut encoder = OpusEncoder::new(16000);
        let initial_bitrate = encoder.current_bitrate();

        encoder.enable_adaptive_bitrate();

        // High packet loss should reduce bitrate
        encoder.adapt_bitrate(15.0, 100);
        assert!(encoder.current_bitrate() < initial_bitrate);

        // Reset
        encoder.disable_adaptive_bitrate();
        assert_eq!(encoder.current_bitrate(), initial_bitrate);
    }

    #[test]
    fn test_codec_manager() {
        let config = OpusConfig::default();
        let mut manager = OpusCodecManager::new(config).unwrap();

        let pcm = vec![100i16; 320];
        let opus = manager.encode(&pcm).unwrap();
        let decoded = manager.decode(&opus).unwrap();

        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_opus_config_default() {
        let config = OpusConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
        assert_eq!(config.bitrate, 28000);
        assert!(config.use_dtx);
        assert!(config.use_fec);
    }
}
