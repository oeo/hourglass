use crate::provider::TimeProvider;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use tokio::time;

/// Production time provider that uses actual system time
#[derive(Debug, Clone, Copy)]
pub struct SystemTimeProvider;

#[async_trait]
impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    
    async fn wait(&self, duration: Duration) {
        if let Ok(std_duration) = duration.to_std() {
            time::sleep(std_duration).await;
        }
    }
    
    async fn wait_until(&self, deadline: DateTime<Utc>) {
        let now = self.now();
        if deadline > now {
            let duration = deadline - now;
            self.wait(duration).await;
        }
    }
    
    fn is_test(&self) -> bool {
        false
    }
}