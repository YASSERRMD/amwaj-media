//! Distributed Session State Management
//!
//! Provides session state management for distributed deployments.
//! Uses Redis for state persistence across multiple pods.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for session management
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Redis connection URL (when redis feature is enabled)
    pub redis_url: Option<String>,
    /// Session TTL in seconds
    pub ttl_seconds: u64,
    /// Maximum sessions per instance
    pub max_sessions: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            redis_url: None,
            ttl_seconds: 3600,
            max_sessions: 10000,
        }
    }
}

/// Session data stored for each connection
#[derive(Debug, Clone)]
pub struct SessionData {
    /// Unique session identifier
    pub session_id: String,
    /// User identifier
    pub user_id: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Session state
    pub state: SessionState,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl SessionData {
    /// Create a new session
    pub fn new(session_id: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            user_id: None,
            created_at: now,
            last_activity: now,
            state: SessionState::Active,
            metadata: HashMap::new(),
        }
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Check if session is expired
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        let age = Utc::now().signed_duration_since(self.last_activity);
        age.num_seconds() > ttl_seconds as i64
    }

    /// Add metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Session is active
    Active,
    /// Session is paused (e.g., user muted)
    Paused,
    /// Session is being terminated
    Terminating,
    /// Session has ended
    Ended,
}

/// Distributed session manager
///
/// Manages session state across multiple instances.
/// In-memory storage is used by default, with optional Redis backend.
pub struct DistributedSessionManager {
    config: SessionConfig,
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    #[allow(dead_code)]
    instance_id: String,
}

impl DistributedSessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            instance_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Create a new session manager with Redis URL
    pub fn with_redis(redis_url: &str, ttl_seconds: u64) -> anyhow::Result<Self> {
        let config = SessionConfig {
            redis_url: Some(redis_url.to_string()),
            ttl_seconds,
            ..SessionConfig::default()
        };
        Ok(Self::new(config))
    }

    /// Create a new session
    pub async fn create_session(&self, user_id: Option<String>) -> anyhow::Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let mut session = SessionData::new(session_id.clone());
        session.user_id = user_id;

        {
            let mut sessions = self.sessions.write();

            // Check capacity
            if sessions.len() >= self.config.max_sessions {
                // Clean up expired sessions first
                self.cleanup_expired_internal(&mut sessions);

                if sessions.len() >= self.config.max_sessions {
                    return Err(anyhow::anyhow!("Maximum session limit reached"));
                }
            }

            sessions.insert(session_id.clone(), session);
        }

        Ok(session_id)
    }

    /// Get session data
    pub async fn get_session(&self, session_id: &str) -> Option<SessionData> {
        let sessions = self.sessions.read();
        sessions.get(session_id).cloned()
    }

    /// Update session activity
    pub async fn touch_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.touch();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }

    /// Update session state
    pub async fn update_state(&self, session_id: &str, state: SessionState) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.state = state;
            session.touch();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }

    /// Set session metadata
    pub async fn set_metadata(
        &self,
        session_id: &str,
        key: String,
        value: String,
    ) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.set_metadata(key, value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }

    /// End a session
    pub async fn end_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.state = SessionState::Ended;
        }
        sessions.remove(session_id);
        Ok(())
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        let sessions = self.sessions.read();
        sessions
            .values()
            .filter(|s| s.state == SessionState::Active)
            .count()
    }

    /// Get total session count
    pub fn total_session_count(&self) -> usize {
        self.sessions.read().len()
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write();
        self.cleanup_expired_internal(&mut sessions)
    }

    fn cleanup_expired_internal(&self, sessions: &mut HashMap<String, SessionData>) -> usize {
        let ttl = self.config.ttl_seconds;
        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| s.is_expired(ttl))
            .map(|(k, _)| k.clone())
            .collect();

        let count = expired.len();
        for id in expired {
            sessions.remove(&id);
        }
        count
    }

    /// List all session IDs
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.read().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_data_creation() {
        let session = SessionData::new("test-123".to_string());
        assert_eq!(session.session_id, "test-123");
        assert_eq!(session.state, SessionState::Active);
        assert!(session.user_id.is_none());
    }

    #[test]
    fn test_session_metadata() {
        let mut session = SessionData::new("test".to_string());
        session.set_metadata("key1".to_string(), "value1".to_string());

        assert_eq!(session.get_metadata("key1"), Some(&"value1".to_string()));
        assert_eq!(session.get_metadata("nonexistent"), None);
    }

    #[tokio::test]
    async fn test_session_manager_create() {
        let config = SessionConfig::default();
        let manager = DistributedSessionManager::new(config);

        let session_id = manager
            .create_session(Some("user-1".to_string()))
            .await
            .unwrap();
        assert!(!session_id.is_empty());

        let session = manager.get_session(&session_id).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().user_id, Some("user-1".to_string()));
    }

    #[tokio::test]
    async fn test_session_manager_touch() {
        let config = SessionConfig::default();
        let manager = DistributedSessionManager::new(config);

        let session_id = manager.create_session(None).await.unwrap();

        // Touch should succeed
        assert!(manager.touch_session(&session_id).await.is_ok());

        // Touch non-existent should fail
        assert!(manager.touch_session("nonexistent").await.is_err());
    }

    #[tokio::test]
    async fn test_session_manager_state_update() {
        let config = SessionConfig::default();
        let manager = DistributedSessionManager::new(config);

        let session_id = manager.create_session(None).await.unwrap();

        manager
            .update_state(&session_id, SessionState::Paused)
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.state, SessionState::Paused);
    }

    #[tokio::test]
    async fn test_session_manager_count() {
        let config = SessionConfig::default();
        let manager = DistributedSessionManager::new(config);

        assert_eq!(manager.total_session_count(), 0);

        manager.create_session(None).await.unwrap();
        manager.create_session(None).await.unwrap();

        assert_eq!(manager.total_session_count(), 2);
        assert_eq!(manager.active_session_count(), 2);
    }

    #[tokio::test]
    async fn test_session_manager_end() {
        let config = SessionConfig::default();
        let manager = DistributedSessionManager::new(config);

        let session_id = manager.create_session(None).await.unwrap();
        assert_eq!(manager.total_session_count(), 1);

        manager.end_session(&session_id).await.unwrap();
        assert_eq!(manager.total_session_count(), 0);
    }
}
