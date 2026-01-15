//! Jitter Buffer for RTP packet reordering and timing

use std::collections::BTreeMap;

/// Jitter buffer to handle out-of-order RTP packets
pub struct JitterBuffer {
    buffer: BTreeMap<u16, Vec<u8>>,
    max_size_ms: u32,
    sample_rate: u32,
    last_sequence: Option<u16>,
    packets_received: u64,
    packets_lost: u64,
}

impl JitterBuffer {
    /// Create a new jitter buffer
    /// 
    /// # Arguments
    /// * `max_size_ms` - Maximum buffer size in milliseconds
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(max_size_ms: u32, sample_rate: u32) -> Self {
        Self {
            buffer: BTreeMap::new(),
            max_size_ms,
            sample_rate,
            last_sequence: None,
            packets_received: 0,
            packets_lost: 0,
        }
    }

    /// Insert a packet into the buffer
    pub fn insert(&mut self, sequence_num: u16, data: Vec<u8>) {
        self.packets_received += 1;
        
        // Check for packet loss
        if let Some(last_seq) = self.last_sequence {
            let expected = last_seq.wrapping_add(1);
            if sequence_num != expected && sequence_num > expected {
                // Packet loss detected
                let lost = sequence_num.wrapping_sub(expected) as u64;
                self.packets_lost += lost;
            }
        }
        
        self.buffer.insert(sequence_num, data);
        
        // Limit buffer size
        let max_packets = self.max_packets();
        while self.buffer.len() > max_packets {
            if let Some((&oldest_seq, _)) = self.buffer.iter().next() {
                self.buffer.remove(&oldest_seq);
            }
        }
    }

    /// Get the next ready frame in sequence order
    pub fn get_ready_frame(&mut self) -> Option<Vec<u8>> {
        if self.buffer.is_empty() {
            return None;
        }

        if let Some((&seq, _)) = self.buffer.iter().next() {
            self.last_sequence = Some(seq);
            self.buffer.remove(&seq)
        } else {
            None
        }
    }

    /// Get all ready frames up to a certain count
    pub fn get_ready_frames(&mut self, max_count: usize) -> Vec<Vec<u8>> {
        let mut frames = Vec::with_capacity(max_count);
        
        for _ in 0..max_count {
            if let Some(frame) = self.get_ready_frame() {
                frames.push(frame);
            } else {
                break;
            }
        }
        
        frames
    }

    /// Check if the buffer has enough data to start playback
    pub fn is_ready(&self, min_packets: usize) -> bool {
        self.buffer.len() >= min_packets
    }

    /// Get current buffer size (number of packets)
    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    /// Get buffer level as a percentage of max capacity
    pub fn level_percent(&self) -> f32 {
        let max = self.max_packets() as f32;
        if max > 0.0 {
            (self.buffer.len() as f32 / max) * 100.0
        } else {
            0.0
        }
    }

    /// Get packet loss ratio
    pub fn packet_loss_ratio(&self) -> f32 {
        if self.packets_received > 0 {
            self.packets_lost as f32 / (self.packets_received + self.packets_lost) as f32
        } else {
            0.0
        }
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.last_sequence = None;
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.packets_received = 0;
        self.packets_lost = 0;
    }

    fn max_packets(&self) -> usize {
        // Assuming 20ms frames
        let frames_per_second = self.sample_rate / 320; // 320 samples per 20ms frame at 16kHz
        ((self.max_size_ms as usize * frames_per_second as usize) / 1000).max(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_retrieve() {
        let mut buffer = JitterBuffer::new(100, 16000);
        
        let data = vec![0x01, 0x02, 0x03];
        buffer.insert(100, data.clone());
        
        let retrieved = buffer.get_ready_frame();
        assert_eq!(retrieved, Some(data));
    }

    #[test]
    fn test_ordering() {
        let mut buffer = JitterBuffer::new(100, 16000);
        
        // Insert out of order
        buffer.insert(102, vec![3]);
        buffer.insert(100, vec![1]);
        buffer.insert(101, vec![2]);
        
        // Should retrieve in order
        assert_eq!(buffer.get_ready_frame(), Some(vec![1]));
        assert_eq!(buffer.get_ready_frame(), Some(vec![2]));
        assert_eq!(buffer.get_ready_frame(), Some(vec![3]));
    }

    #[test]
    fn test_empty_buffer() {
        let mut buffer = JitterBuffer::new(100, 16000);
        assert_eq!(buffer.get_ready_frame(), None);
        assert_eq!(buffer.size(), 0);
    }

    #[test]
    fn test_is_ready() {
        let mut buffer = JitterBuffer::new(100, 16000);
        
        assert!(!buffer.is_ready(3));
        
        buffer.insert(1, vec![1]);
        buffer.insert(2, vec![2]);
        buffer.insert(3, vec![3]);
        
        assert!(buffer.is_ready(3));
    }
}
