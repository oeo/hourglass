# Hourglass

A time abstraction crate for Rust that allows you to test time-dependent code by manipulating time in tests while maintaining zero overhead in production.

## Features

- **Zero overhead** in production - thin wrapper around system time
- **Time manipulation** in tests - advance time, set specific times, track wait calls
- **Async support** - works with tokio's async runtime  
- **Type safety** - can't accidentally manipulate time in production
- **Test isolation** - each test gets its own time control

## Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
hourglass = "0.1.0"
```

## Usage

### Production Code

Write your time-dependent code using `SafeTimeProvider`:

```rust
use hourglass::{SafeTimeProvider, TimeSource};
use chrono::Duration;

async fn daily_task(time_provider: &SafeTimeProvider) {
    loop {
        println!("Task executed at: {}", time_provider.now());
        
        // Wait for 24 hours
        time_provider.wait(Duration::days(1)).await;
    }
}

#[tokio::main]
async fn main() {
    // In production, use system time
    let time = SafeTimeProvider::new(TimeSource::System);
    daily_task(&time).await;
}
```

### Testing

Test time-dependent code by controlling time:

```rust
#[tokio::test]
async fn test_daily_task() {
    // Create a test time provider starting at a specific time
    let time = SafeTimeProvider::new(
        TimeSource::Test("2024-01-01T00:00:00Z".parse().unwrap())
    );
    
    // Get time control for the test
    let control = time.test_control().expect("Should be in test mode");
    
    // Start the task
    let task_handle = tokio::spawn(daily_task(time.clone()));
    
    // Advance time by 3 days instantly
    control.advance(Duration::days(3));
    
    // Verify the task executed 3 times
    assert_eq!(control.wait_call_count(), 3);
    assert_eq!(control.total_waited(), Duration::days(3));
}
```

## Examples

### Interest Calculation

Calculate compound interest with testable time:

```rust
use hourglass::{SafeTimeProvider, TimeSource};
use chrono::{DateTime, Duration, Utc};

struct InterestCalculator {
    time_provider: SafeTimeProvider,
}

impl InterestCalculator {
    fn calculate_interest(
        &self,
        principal: f64,
        rate: f64,
        last_accrual: DateTime<Utc>,
    ) -> f64 {
        let now = self.time_provider.now();
        let days = (now - last_accrual).num_days() as f64;
        principal * rate * days / 365.0
    }
}
```

### Scheduled Jobs

Run scheduled jobs:

```rust
async fn run_hourly_job(time: &SafeTimeProvider) {
    loop {
        process_job().await;
        time.wait(Duration::hours(1)).await;
    }
}

#[test]
async fn test_hourly_job() {
    let time = SafeTimeProvider::new(TimeSource::TestNow);
    let control = time.test_control().unwrap();
    
    // Simulate 24 hours instantly
    for _ in 0..24 {
        control.advance(Duration::hours(1));
    }
}
```

### Running Examples

The crate includes several example applications demonstrating different use cases:

```bash
# Basic time manipulation
cargo run --example basic_usage

# Async operations with time control
cargo run --example async_wait

# Interest calculation simulation
cargo run --example interest_calc

# Loan accrual simulation (daily interest, monthly cycles)
cargo run --example loan_accrual

# Collateral margin monitoring (CVL ratios, liquidation)
cargo run --example margin_monitoring

# Loan lifecycle edge cases (month-end handling, overdue detection)
cargo run --example loan_lifecycle
```

All examples use test mode by default to demonstrate time manipulation. To run in production mode, set:
```bash
TIME_SOURCE=system cargo run --example basic_usage
```

## Time Sources

- `TimeSource::System` - Uses actual system time (production)
- `TimeSource::Test(start_time)` - Test mode starting at specific time
- `TimeSource::TestNow` - Test mode starting at current system time

### Environment Variables

Configure time source via environment:
- `TIME_SOURCE=system` (default) or `TIME_SOURCE=test`  
- `TIME_START=2024-01-01T00:00:00Z` (RFC3339 format for test mode)

## API Reference

### SafeTimeProvider

The main interface for time operations:

- `now()` - Get current time
- `wait(duration)` - Async wait for duration
- `wait_until(deadline)` - Async wait until specific time
- `is_test_mode()` - Check if running in test mode
- `test_control()` - Get time control (test mode only)

### TimeControl

Test-only time manipulation (via `test_control()`):

- `advance(duration)` - Advance time forward
- `set(time)` - Set time to specific value
- `total_waited()` - Get total duration waited
- `wait_call_count()` - Get number of wait calls
- `reset_wait_tracking()` - Reset wait statistics

## Usage Notes

1. **Dependency Injection** - Pass `SafeTimeProvider` to your structs/functions
2. **Avoid Direct Time Access** - Use the provider instead of `Utc::now()`
3. **Test Time Boundaries** - Test edge cases like midnight, month boundaries
4. **Isolated Tests** - Each test should have its own time provider

## Contributing

Contributions accepted via Pull Request.