use hourglass::{SafeTimeProvider, TimeSource};
use chrono::Duration;

#[tokio::main]
async fn main() {
    println!("=== Hourglass Basic Usage Example ===\n");
    
    // Production usage
    println!("1. Production Mode (System Time):");
    let prod_time = SafeTimeProvider::new(TimeSource::System);
    println!("   Current time: {}", prod_time.now());
    println!("   Is test mode: {}", prod_time.is_test_mode());
    println!("   Waiting 2 seconds...");
    let start = prod_time.now();
    prod_time.wait(Duration::seconds(2)).await;
    let elapsed = prod_time.now() - start;
    println!("   Elapsed time: {} seconds\n", elapsed.num_milliseconds() as f64 / 1000.0);
    
    // Test usage
    println!("2. Test Mode (Controlled Time):");
    let test_time = SafeTimeProvider::new(
        TimeSource::Test("2024-01-01T00:00:00Z".parse().unwrap())
    );
    println!("   Starting time: {}", test_time.now());
    println!("   Is test mode: {}", test_time.is_test_mode());
    
    // Get time control
    let control = test_time.test_control().expect("Should have time control in test mode");
    
    // Simulate waiting without actually waiting
    println!("   'Waiting' 30 days...");
    let start = test_time.now();
    test_time.wait(Duration::days(30)).await;
    let end = test_time.now();
    
    println!("   End time: {}", end);
    println!("   Time advanced: {} days", (end - start).num_days());
    println!("   Total time 'waited': {} days", control.total_waited().num_days());
    println!("   Wait calls made: {}\n", control.wait_call_count());
    
    // Test time manipulation
    println!("3. Time Manipulation in Tests:");
    control.set("2024-12-25T00:00:00Z".parse().unwrap());
    println!("   Jumped to: {}", test_time.now());
    
    control.advance(Duration::hours(6));
    println!("   Advanced 6 hours to: {}", test_time.now());
    
    // Reset tracking
    control.reset_wait_tracking();
    println!("   Reset wait tracking");
    println!("   Total waited after reset: {} seconds", control.total_waited().num_seconds());
}