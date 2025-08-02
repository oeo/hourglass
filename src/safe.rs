use crate::config::TimeSource;
use crate::control::TimeControl;
use crate::provider::SharedTimeProvider;
use crate::test::TestTimeProvider;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

/// Production-safe time provider wrapper that prevents accidental time manipulation
pub struct SafeTimeProvider {
    inner: SharedTimeProvider,
    test_provider: Option<Arc<TestTimeProvider>>,
}

impl SafeTimeProvider {
    /// Create a new SafeTimeProvider from a TimeSource
    pub fn new(source: TimeSource) -> Self {
        match source {
            TimeSource::System => Self {
                inner: Arc::new(crate::system::SystemTimeProvider),
                test_provider: None,
            },
            TimeSource::Test(start) => {
                let test_provider = Arc::new(TestTimeProvider::new(start));
                Self {
                    inner: test_provider.clone() as SharedTimeProvider,
                    test_provider: Some(test_provider),
                }
            },
            TimeSource::TestNow => {
                let test_provider = Arc::new(TestTimeProvider::new_at_now());
                Self {
                    inner: test_provider.clone() as SharedTimeProvider,
                    test_provider: Some(test_provider),
                }
            },
        }
    }
    
    /// Create from an existing test provider (mainly for testing)
    pub fn new_from_test_provider(provider: Arc<TestTimeProvider>) -> Self {
        Self {
            inner: provider.clone() as SharedTimeProvider,
            test_provider: Some(provider),
        }
    }
    
    /// Get the current time
    pub fn now(&self) -> DateTime<Utc> {
        self.inner.now()
    }
    
    /// Wait for the specified duration
    pub async fn wait(&self, duration: Duration) {
        self.inner.wait(duration).await
    }
    
    /// Wait until the specified deadline
    pub async fn wait_until(&self, deadline: DateTime<Utc>) {
        self.inner.wait_until(deadline).await
    }
    
    /// Check if running in test mode
    pub fn is_test_mode(&self) -> bool {
        self.inner.is_test()
    }
    
    /// Get time control for tests (returns None in production)
    /// 
    /// This method returns a TimeControl guard that allows time manipulation
    /// only when using a test time provider.
    pub fn test_control(&self) -> Option<TimeControl> {
        self.test_provider
            .as_ref()
            .map(|provider| TimeControl::new(provider.clone()))
    }
}

impl Clone for SafeTimeProvider {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            test_provider: self.test_provider.clone(),
        }
    }
}