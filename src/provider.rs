use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

/// Type alias for a shared time provider
pub type SharedTimeProvider = Arc<dyn TimeProvider>;

/// Core trait for time providers
#[async_trait]
pub trait TimeProvider: Send + Sync {
    /// Get the current time
    fn now(&self) -> DateTime<Utc>;
    
    /// Wait for the specified duration
    async fn wait(&self, duration: Duration);
    
    /// Wait until the specified deadline
    async fn wait_until(&self, deadline: DateTime<Utc>);
    
    /// Check if this is a test provider
    fn is_test(&self) -> bool;
}