use crate::test::TestTimeProvider;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

/// A guard that provides safe access to time manipulation methods in tests
pub struct TimeControl {
    provider: Arc<TestTimeProvider>,
}

impl TimeControl {
    /// Create a new TimeControl from a TestTimeProvider
    pub(crate) fn new(provider: Arc<TestTimeProvider>) -> Self {
        Self { provider }
    }
    
    /// Advance time by the specified duration
    pub fn advance(&self, duration: Duration) {
        self.provider.advance(duration);
    }
    
    /// Set time to a specific value
    pub fn set(&self, time: DateTime<Utc>) {
        self.provider.set(time);
    }
    
    /// Get the total duration waited since creation or last reset
    pub fn total_waited(&self) -> Duration {
        self.provider.total_waited()
    }
    
    /// Reset wait tracking statistics
    pub fn reset_wait_tracking(&self) {
        self.provider.reset_wait_tracking();
    }
    
    /// Get the number of wait calls since creation or last reset
    pub fn wait_call_count(&self) -> usize {
        self.provider.wait_call_count()
    }
}

impl std::fmt::Debug for TimeControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimeControl")
            .field("total_waited", &self.total_waited())
            .field("wait_call_count", &self.wait_call_count())
            .finish()
    }
}