use hourglass::{SafeTimeProvider, TimeSource};
use chrono::{Duration, DateTime, Utc};
use tokio::task::JoinHandle;

/// Simulated async task that waits for a specific time
async fn scheduled_task(time: SafeTimeProvider, task_name: &str, wait_duration: Duration) -> String {
    let start = time.now();
    println!("[{}] Starting at: {}", task_name, start);
    
    // Wait for the specified duration
    time.wait(wait_duration).await;
    
    let end = time.now();
    println!("[{}] Completed at: {}", task_name, end);
    
    format!("{} completed after {} seconds", task_name, (end - start).num_seconds())
}

/// Simulated task that waits until a specific time
async fn wait_until_task(time: SafeTimeProvider, deadline: DateTime<Utc>) -> String {
    let start = time.now();
    println!("[WaitUntil] Starting at: {}, waiting until: {}", start, deadline);
    
    time.wait_until(deadline).await;
    
    let end = time.now();
    println!("[WaitUntil] Reached deadline at: {}", end);
    
    format!("Waited {} seconds to reach deadline", (end - start).num_seconds())
}

#[tokio::main]
async fn main() {
    println!("=== Hourglass Async Wait Example ===\n");
    
    // Test mode demonstration
    println!("Test Mode - Time Manipulation with Concurrent Tasks:");
    
    let test_time = SafeTimeProvider::new(
        TimeSource::Test("2024-01-01T00:00:00Z".parse().unwrap())
    );
    let control = test_time.test_control().expect("Should have time control");
    
    // Spawn multiple concurrent tasks
    let time1 = test_time.clone();
    let task1: JoinHandle<String> = tokio::spawn(async move {
        scheduled_task(time1, "Task1", Duration::hours(1)).await
    });
    
    let time2 = test_time.clone();
    let task2: JoinHandle<String> = tokio::spawn(async move {
        scheduled_task(time2, "Task2", Duration::hours(2)).await
    });
    
    let time3 = test_time.clone();
    let deadline = test_time.now() + Duration::hours(3);
    let task3: JoinHandle<String> = tokio::spawn(async move {
        wait_until_task(time3, deadline).await
    });
    
    // Let tasks start
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    
    println!("\nAdvancing time by 1 hour...");
    control.advance(Duration::hours(1));
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    
    println!("\nAdvancing time by 1 more hour...");
    control.advance(Duration::hours(1));
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    
    println!("\nJumping to the 3-hour mark...");
    control.set("2024-01-01T03:00:00Z".parse().unwrap());
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    
    // Collect results
    let results = tokio::join!(task1, task2, task3);
    
    println!("\n=== Results ===");
    println!("Task 1: {}", results.0.unwrap());
    println!("Task 2: {}", results.1.unwrap());
    println!("Task 3: {}", results.2.unwrap());
    
    println!("\n=== Statistics ===");
    println!("Final time: {}", test_time.now());
    println!("Total time waited across all tasks: {} hours", 
             control.total_waited().num_hours());
    println!("Total wait calls: {}", control.wait_call_count());
    
    // Production mode comparison
    println!("\n=== Production Mode Comparison ===");
    println!("In production mode, the same tasks would have actually waited:");
    println!("- Task 1: 1 hour");
    println!("- Task 2: 2 hours");
    println!("- Task 3: 3 hours");
    println!("Total real time: 3 hours (running concurrently)");
    println!("\nIn test mode, we simulated this in milliseconds!");
}