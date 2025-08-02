use crate::provider::SharedTimeProvider;
use crate::system::SystemTimeProvider;
use crate::test::TestTimeProvider;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Time source configuration for different environments
#[derive(Debug, Clone)]
pub enum TimeSource {
    /// Use system time (production)
    System,
    /// Use test time with initial timestamp
    Test(DateTime<Utc>),
    /// Use test time starting at current system time
    TestNow,
}

impl TimeSource {
    /// Create from environment variables
    /// - TIME_SOURCE: "system" (default) or "test"
    /// - TIME_START: RFC3339 timestamp for test mode start time
    pub fn from_env() -> Self {
        match std::env::var("TIME_SOURCE").as_deref() {
            Ok("test") => {
                if let Ok(start_str) = std::env::var("TIME_START") {
                    if let Ok(start_time) = DateTime::parse_from_rfc3339(&start_str) {
                        TimeSource::Test(start_time.with_timezone(&Utc))
                    } else {
                        eprintln!("Invalid TIME_START format, using current time");
                        TimeSource::TestNow
                    }
                } else {
                    TimeSource::TestNow
                }
            }
            _ => TimeSource::System,
        }
    }
    
    /// Convert to a time provider instance
    pub fn into_provider(self) -> SharedTimeProvider {
        match self {
            TimeSource::System => Arc::new(SystemTimeProvider),
            TimeSource::Test(start) => Arc::new(TestTimeProvider::new(start)),
            TimeSource::TestNow => Arc::new(TestTimeProvider::new_at_now()),
        }
    }
}

impl Default for TimeSource {
    fn default() -> Self {
        TimeSource::System
    }
}