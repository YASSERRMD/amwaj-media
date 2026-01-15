//! ICE (Interactive Connectivity Establishment) module
//!
//! Provides ICE candidate gathering and connectivity checking
//! for WebRTC NAT traversal.

use std::net::SocketAddr;

/// ICE candidate types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateType {
    /// Host candidate (local IP)
    Host,
    /// Server reflexive candidate (STUN result)
    ServerReflexive,
    /// Peer reflexive candidate (discovered during check)
    PeerReflexive,
    /// Relay candidate (TURN server)
    Relay,
}

impl std::fmt::Display for CandidateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CandidateType::Host => write!(f, "host"),
            CandidateType::ServerReflexive => write!(f, "srflx"),
            CandidateType::PeerReflexive => write!(f, "prflx"),
            CandidateType::Relay => write!(f, "relay"),
        }
    }
}

/// ICE candidate
#[derive(Debug, Clone)]
pub struct IceCandidate {
    /// Foundation (hash of type, base IP, and STUN server)
    pub foundation: String,
    /// Component ID (1 = RTP, 2 = RTCP)
    pub component: u8,
    /// Transport protocol
    pub transport: String,
    /// Priority value
    pub priority: u32,
    /// Connection address
    pub address: SocketAddr,
    /// Candidate type
    pub candidate_type: CandidateType,
    /// Related address (for srflx/relay)
    pub related_address: Option<SocketAddr>,
}

impl IceCandidate {
    /// Create a host candidate
    pub fn host(address: SocketAddr, component: u8) -> Self {
        Self {
            foundation: format!("host-{}", uuid::Uuid::new_v4()),
            component,
            transport: "UDP".to_string(),
            priority: Self::calculate_priority(CandidateType::Host, component),
            address,
            candidate_type: CandidateType::Host,
            related_address: None,
        }
    }

    /// Create a server reflexive candidate
    pub fn server_reflexive(address: SocketAddr, base_address: SocketAddr, component: u8) -> Self {
        Self {
            foundation: format!("srflx-{}", uuid::Uuid::new_v4()),
            component,
            transport: "UDP".to_string(),
            priority: Self::calculate_priority(CandidateType::ServerReflexive, component),
            address,
            candidate_type: CandidateType::ServerReflexive,
            related_address: Some(base_address),
        }
    }

    /// Create a relay candidate
    pub fn relay(address: SocketAddr, base_address: SocketAddr, component: u8) -> Self {
        Self {
            foundation: format!("relay-{}", uuid::Uuid::new_v4()),
            component,
            transport: "UDP".to_string(),
            priority: Self::calculate_priority(CandidateType::Relay, component),
            address,
            candidate_type: CandidateType::Relay,
            related_address: Some(base_address),
        }
    }

    /// Calculate priority based on type and component
    fn calculate_priority(candidate_type: CandidateType, component: u8) -> u32 {
        let type_preference: u32 = match candidate_type {
            CandidateType::Host => 126,
            CandidateType::PeerReflexive => 110,
            CandidateType::ServerReflexive => 100,
            CandidateType::Relay => 0,
        };
        let local_preference: u32 = 65535;
        let component_id = component as u32;

        (type_preference << 24) + (local_preference << 8) + (256 - component_id)
    }

    /// Format as SDP attribute
    pub fn to_sdp(&self) -> String {
        let mut sdp = format!(
            "candidate:{} {} {} {} {} {} typ {}",
            self.foundation,
            self.component,
            self.transport,
            self.priority,
            self.address.ip(),
            self.address.port(),
            self.candidate_type
        );

        if let Some(related) = &self.related_address {
            sdp.push_str(&format!(" raddr {} rport {}", related.ip(), related.port()));
        }

        sdp
    }
}

/// TURN server configuration
#[derive(Debug, Clone)]
pub struct TurnServerConfig {
    /// Server URL
    pub url: String,
    /// Username for authentication
    pub username: String,
    /// Password/credential
    pub credential: String,
}

/// ICE gatherer for collecting candidates
pub struct IceGatherer {
    stun_servers: Vec<String>,
    turn_servers: Vec<TurnServerConfig>,
    candidates: Vec<IceCandidate>,
    gathering_complete: bool,
}

impl IceGatherer {
    /// Create a new ICE gatherer
    pub fn new(stun_servers: Vec<String>, turn_servers: Vec<TurnServerConfig>) -> Self {
        Self {
            stun_servers,
            turn_servers,
            candidates: Vec::new(),
            gathering_complete: false,
        }
    }

    /// Create with default STUN servers
    pub fn with_defaults() -> Self {
        Self::new(
            vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            Vec::new(),
        )
    }

    /// Gather ICE candidates
    pub async fn gather(&mut self) -> anyhow::Result<Vec<IceCandidate>> {
        self.candidates.clear();
        self.gathering_complete = false;

        // Gather host candidates
        self.gather_host_candidates().await?;

        // Gather server reflexive candidates (STUN)
        if !self.stun_servers.is_empty() {
            self.gather_srflx_candidates().await?;
        }

        // Gather relay candidates (TURN)
        if !self.turn_servers.is_empty() {
            self.gather_relay_candidates().await?;
        }

        self.gathering_complete = true;
        Ok(self.candidates.clone())
    }

    async fn gather_host_candidates(&mut self) -> anyhow::Result<()> {
        // TODO: Enumerate local interfaces
        // For now, add a placeholder host candidate
        let addr: SocketAddr = "0.0.0.0:0".parse()?;
        self.candidates.push(IceCandidate::host(addr, 1));
        Ok(())
    }

    async fn gather_srflx_candidates(&mut self) -> anyhow::Result<()> {
        // TODO: Perform STUN binding requests
        // For now, this is a stub
        tracing::debug!("STUN gathering from {:?}", self.stun_servers);
        Ok(())
    }

    async fn gather_relay_candidates(&mut self) -> anyhow::Result<()> {
        // TODO: Perform TURN allocations
        // For now, this is a stub
        tracing::debug!("TURN allocation from {:?}", self.turn_servers.len());
        Ok(())
    }

    /// Get gathered candidates
    pub fn candidates(&self) -> &[IceCandidate] {
        &self.candidates
    }

    /// Check if gathering is complete
    pub fn is_complete(&self) -> bool {
        self.gathering_complete
    }

    /// Add a remote candidate for connectivity checking
    pub fn add_remote_candidate(&mut self, _candidate: IceCandidate) {
        // TODO: Add to remote candidate list for pair checking
    }
}

/// STUN client for NAT discovery
pub struct StunClient {
    server_addr: String,
}

impl StunClient {
    /// Create a new STUN client
    pub fn new(server_addr: &str) -> Self {
        Self {
            server_addr: server_addr.to_string(),
        }
    }

    /// Discover mapped address via STUN
    pub async fn discover_mapped_address(&self) -> anyhow::Result<SocketAddr> {
        // TODO: Implement actual STUN binding request
        // For now, return a placeholder
        tracing::debug!("STUN discovery to {}", self.server_addr);
        Ok("0.0.0.0:0".parse()?)
    }

    /// Get server address
    pub fn server_addr(&self) -> &str {
        &self.server_addr
    }
}

/// TURN client for relay allocation
pub struct TurnClient {
    config: TurnServerConfig,
    allocated: bool,
    relay_address: Option<SocketAddr>,
}

impl TurnClient {
    /// Create a new TURN client
    pub fn new(config: TurnServerConfig) -> Self {
        Self {
            config,
            allocated: false,
            relay_address: None,
        }
    }

    /// Allocate a relay address
    pub async fn allocate(&mut self) -> anyhow::Result<SocketAddr> {
        // TODO: Implement actual TURN allocation
        tracing::debug!("TURN allocation to {}", self.config.url);
        self.allocated = true;
        let addr: SocketAddr = "0.0.0.0:0".parse()?;
        self.relay_address = Some(addr);
        Ok(addr)
    }

    /// Refresh the allocation
    pub async fn refresh(&mut self) -> anyhow::Result<()> {
        if !self.allocated {
            return Err(anyhow::anyhow!("No active allocation"));
        }
        // TODO: Send TURN refresh
        Ok(())
    }

    /// Release the allocation
    pub async fn release(&mut self) -> anyhow::Result<()> {
        self.allocated = false;
        self.relay_address = None;
        Ok(())
    }

    /// Check if allocated
    pub fn is_allocated(&self) -> bool {
        self.allocated
    }

    /// Get relay address
    pub fn relay_address(&self) -> Option<SocketAddr> {
        self.relay_address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_types() {
        assert_eq!(format!("{}", CandidateType::Host), "host");
        assert_eq!(format!("{}", CandidateType::ServerReflexive), "srflx");
        assert_eq!(format!("{}", CandidateType::Relay), "relay");
    }

    #[test]
    fn test_host_candidate() {
        let addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let candidate = IceCandidate::host(addr, 1);

        assert_eq!(candidate.candidate_type, CandidateType::Host);
        assert_eq!(candidate.address, addr);
        assert!(candidate.related_address.is_none());
    }

    #[test]
    fn test_srflx_candidate() {
        let public: SocketAddr = "203.0.113.1:5000".parse().unwrap();
        let local: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let candidate = IceCandidate::server_reflexive(public, local, 1);

        assert_eq!(candidate.candidate_type, CandidateType::ServerReflexive);
        assert_eq!(candidate.address, public);
        assert_eq!(candidate.related_address, Some(local));
    }

    #[test]
    fn test_candidate_priority() {
        let host = IceCandidate::host("192.168.1.1:5000".parse().unwrap(), 1);
        let srflx = IceCandidate::server_reflexive(
            "203.0.113.1:5000".parse().unwrap(),
            "192.168.1.1:5000".parse().unwrap(),
            1,
        );
        let relay = IceCandidate::relay(
            "198.51.100.1:5000".parse().unwrap(),
            "192.168.1.1:5000".parse().unwrap(),
            1,
        );

        assert!(host.priority > srflx.priority);
        assert!(srflx.priority > relay.priority);
    }

    #[test]
    fn test_candidate_sdp() {
        let addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let candidate = IceCandidate::host(addr, 1);
        let sdp = candidate.to_sdp();

        assert!(sdp.contains("candidate:"));
        assert!(sdp.contains("typ host"));
        assert!(sdp.contains("192.168.1.100"));
    }

    #[tokio::test]
    async fn test_ice_gatherer() {
        let mut gatherer = IceGatherer::with_defaults();
        let candidates = gatherer.gather().await.unwrap();

        assert!(!candidates.is_empty());
        assert!(gatherer.is_complete());
    }

    #[test]
    fn test_stun_client() {
        let client = StunClient::new("stun.l.google.com:19302");
        assert_eq!(client.server_addr(), "stun.l.google.com:19302");
    }

    #[tokio::test]
    async fn test_turn_client() {
        let config = TurnServerConfig {
            url: "turn:turn.example.com:3478".to_string(),
            username: "user".to_string(),
            credential: "pass".to_string(),
        };

        let mut client = TurnClient::new(config);
        assert!(!client.is_allocated());

        client.allocate().await.unwrap();
        assert!(client.is_allocated());
        assert!(client.relay_address().is_some());

        client.release().await.unwrap();
        assert!(!client.is_allocated());
    }
}
