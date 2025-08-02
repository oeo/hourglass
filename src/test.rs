use crate::provider::TimeProvider;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use std::sync::Arc;

/// Test time provider that allows time manipulation
pub struct TestTimeProvider {
    state: Arc<RwLock<TestState>>,
}

#[derive(Debug)]
struct TestState {
    current_time: DateTime<Utc>,
    total_waited: Duration,
    wait_call_count: usize,
}

impl TestTimeProvider {
    /// Create a new test provider at the specified time
    pub fn new(start: DateTime<Utc>) -> Self {
        Self {
            state: Arc::new(RwLock::new(TestState {
                current_time: start,
                total_waited: Duration::zero(),
                wait_call_count: 0,
            })),
        }
    }
    
    /// Create a new test provider at the current system time
    pub fn new_at_now() -> Self {
        Self::new(Utc::now())
    }
    
    /// Advance time by the specified duration
    pub fn advance(&self, duration: Duration) {
        let mut state = self.state.write();
        state.current_time = state.current_time + duration;
    }
    
    /// Set time to a specific value
    pub fn set(&self, time: DateTime<Utc>) {
        let mut state = self.state.write();
        state.current_time = time;
    }
    
    /// Get the total duration waited
    pub fn total_waited(&self) -> Duration {
        self.state.read().total_waited
    }
    
    /// Reset wait tracking statistics
    pub fn reset_wait_tracking(&self) {
        let mut state = self.state.write();
        state.total_waited = Duration::zero();
        state.wait_call_count = 0;
    }
    
    /// Get the number of wait calls
    pub fn wait_call_count(&self) -> usize {
        self.state.read().wait_call_count
    }
}

#[async_trait]
impl TimeProvider for TestTimeProvider {
    fn now(&self) -> DateTime<Utc> {
        self.state.read().current_time
    }
    
    async fn wait(&self, duration: Duration) {
        {
            let mut state = self.state.write();
            state.current_time = state.current_time + duration;
            state.total_waited = state.total_waited + duration;
            state.wait_call_count += 1;
        } // Lock is dropped here
        
        // Yield to allow other tasks to run
        tokio::task::yield_now().await;
    }
    
    async fn wait_until(&self, deadline: DateTime<Utc>) {
        let now = self.now();
        if deadline > now {
            let duration = deadline - now;
            self.wait(duration).await;
        }
    }
    
    fn is_test(&self) -> bool {
        true
    }
}