//! RTP Packet Handler

/// RTP Packet structure according to RFC 3550
#[derive(Debug, Clone)]
pub struct RtpPacket {
    pub version: u8,
    pub padding: bool,
    pub extension: bool,
    pub csrc_count: u8,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub payload: Vec<u8>,
}

impl RtpPacket {
    /// Parse an RTP packet from raw bytes
    pub fn parse(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 12 {
            return Err(anyhow::anyhow!("RTP packet too short: {} bytes", data.len()));
        }

        let version = (data[0] >> 6) & 0x3;
        if version != 2 {
            return Err(anyhow::anyhow!("Invalid RTP version: {}", version));
        }

        let padding = (data[0] & 0x20) != 0;
        let extension = (data[0] & 0x10) != 0;
        let csrc_count = data[0] & 0xF;
        let marker = (data[1] & 0x80) != 0;
        let payload_type = data[1] & 0x7F;
        let sequence_number = u16::from_be_bytes([data[2], data[3]]);
        let timestamp = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ssrc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        let header_size = 12 + (csrc_count as usize * 4);
        
        if data.len() < header_size {
            return Err(anyhow::anyhow!("RTP packet header incomplete"));
        }

        let payload_start = header_size;
        let payload = data[payload_start..].to_vec();

        Ok(Self {
            version,
            padding,
            extension,
            csrc_count,
            marker,
            payload_type,
            sequence_number,
            timestamp,
            ssrc,
            payload,
        })
    }

    /// Serialize the RTP packet back to bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(12 + self.payload.len());
        
        // First byte: V=2, P, X, CC
        let byte0 = (self.version << 6) 
            | if self.padding { 0x20 } else { 0 }
            | if self.extension { 0x10 } else { 0 }
            | (self.csrc_count & 0x0F);
        data.push(byte0);
        
        // Second byte: M, PT
        let byte1 = if self.marker { 0x80 } else { 0 } | (self.payload_type & 0x7F);
        data.push(byte1);
        
        // Sequence number (2 bytes)
        data.extend_from_slice(&self.sequence_number.to_be_bytes());
        
        // Timestamp (4 bytes)
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        
        // SSRC (4 bytes)
        data.extend_from_slice(&self.ssrc.to_be_bytes());
        
        // Payload
        data.extend_from_slice(&self.payload);
        
        data
    }

    /// Check if this is an Opus audio packet (payload type 111 is common for Opus)
    pub fn is_opus(&self) -> bool {
        self.payload_type == 111 || self.payload_type == 96
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_packet() {
        let data = vec![
            0x80, 0x78, 0x00, 0x01,  // Version=2, marker=0, PT=120, seq=1
            0x00, 0x00, 0x00, 0x00,  // Timestamp
            0x00, 0x00, 0x00, 0x01,  // SSRC
            0xAA, 0xBB, 0xCC, 0xDD,  // Payload
        ];

        let packet = RtpPacket::parse(&data).expect("Failed to parse RTP packet");
        assert_eq!(packet.version, 2);
        assert_eq!(packet.sequence_number, 1);
        assert_eq!(packet.payload_type, 120);
        assert_eq!(packet.payload.len(), 4);
    }

    #[test]
    fn test_parse_too_short() {
        let data = vec![0x80, 0x78, 0x00];
        assert!(RtpPacket::parse(&data).is_err());
    }

    #[test]
    fn test_serialize_roundtrip() {
        let original = RtpPacket {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: true,
            payload_type: 111,
            sequence_number: 1234,
            timestamp: 5678,
            ssrc: 9012,
            payload: vec![1, 2, 3, 4],
        };

        let serialized = original.serialize();
        let parsed = RtpPacket::parse(&serialized).expect("Failed to parse");
        
        assert_eq!(parsed.version, original.version);
        assert_eq!(parsed.marker, original.marker);
        assert_eq!(parsed.payload_type, original.payload_type);
        assert_eq!(parsed.sequence_number, original.sequence_number);
        assert_eq!(parsed.timestamp, original.timestamp);
        assert_eq!(parsed.ssrc, original.ssrc);
        assert_eq!(parsed.payload, original.payload);
    }
}
