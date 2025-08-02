use hourglass_rs::{SafeTimeProvider, TimeSource};
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

/// Interest calculator service
struct InterestCalculator {
    time_provider: SafeTimeProvider,
}

impl InterestCalculator {
    pub fn new(time_provider: SafeTimeProvider) -> Self {
        Self { time_provider }
    }
    
    /// Calculate simple interest accrued between last accrual and now
    pub fn calculate_interest(
        &self,
        principal: f64,
        annual_rate: f64,
        last_accrual: DateTime<Utc>,
    ) -> (f64, DateTime<Utc>) {
        let now = self.time_provider.now();
        let days = (now - last_accrual).num_days() as f64;
        let interest = principal * annual_rate * days / 365.0;
        (interest, now)
    }
    
    /// Calculate compound interest
    pub fn calculate_compound_interest(
        &self,
        principal: f64,
        annual_rate: f64,
        compounds_per_year: u32,
        last_accrual: DateTime<Utc>,
    ) -> (f64, f64, DateTime<Utc>) {
        let now = self.time_provider.now();
        let years = (now - last_accrual).num_days() as f64 / 365.0;
        let rate_per_period = annual_rate / compounds_per_year as f64;
        let periods = compounds_per_year as f64 * years;
        
        let final_amount = principal * (1.0 + rate_per_period).powf(periods);
        let interest = final_amount - principal;
        
        (final_amount, interest, now)
    }
}

/// Background job scheduler for interest accruals
struct InterestAccrualScheduler {
    time_provider: SafeTimeProvider,
    calculator: Arc<InterestCalculator>,
}

impl InterestAccrualScheduler {
    pub fn new(time_provider: SafeTimeProvider) -> Self {
        let calculator = Arc::new(InterestCalculator::new(time_provider.clone()));
        Self {
            time_provider,
            calculator,
        }
    }
    
    /// Run daily interest accrual at 2 AM
    pub async fn run_daily_accruals(&self, accounts: &mut Vec<Account>) {
        loop {
            // Calculate next 2 AM
            let now = self.time_provider.now();
            let tomorrow = now.date_naive().succ_opt().unwrap();
            let next_run = tomorrow.and_hms_opt(2, 0, 0).unwrap().and_utc();
            
            println!("Next accrual scheduled for: {}", next_run);
            
            // Wait until 2 AM
            self.time_provider.wait_until(next_run).await;
            
            println!("Running interest accrual at: {}", self.time_provider.now());
            
            // Process each account
            for account in accounts.iter_mut() {
                let (interest, accrual_time) = self.calculator.calculate_interest(
                    account.balance,
                    account.interest_rate,
                    account.last_accrual,
                );
                
                account.balance += interest;
                account.last_accrual = accrual_time;
                account.total_interest += interest;
                
                println!("  Account {}: Accrued ${:.2} interest", account.id, interest);
            }
            
            println!("Daily accrual complete\n");
        }
    }
}

#[derive(Debug, Clone)]
struct Account {
    id: String,
    balance: f64,
    interest_rate: f64,
    last_accrual: DateTime<Utc>,
    total_interest: f64,
}

#[tokio::main]
async fn main() {
    println!("=== Hourglass Banking Interest Calculator Example ===\n");
    
    // Test mode for demonstration
    let time = SafeTimeProvider::new(
        TimeSource::Test("2024-01-01T00:00:00Z".parse().unwrap())
    );
    let control = time.test_control().expect("Should have time control");
    
    // Create some test accounts
    let accounts = vec![
        Account {
            id: "SAV-001".to_string(),
            balance: 10_000.0,
            interest_rate: 0.02, // 2% APY
            last_accrual: time.now(),
            total_interest: 0.0,
        },
        Account {
            id: "SAV-002".to_string(),
            balance: 50_000.0,
            interest_rate: 0.025, // 2.5% APY
            last_accrual: time.now(),
            total_interest: 0.0,
        },
        Account {
            id: "SAV-003".to_string(),
            balance: 100_000.0,
            interest_rate: 0.03, // 3% APY (premium account)
            last_accrual: time.now(),
            total_interest: 0.0,
        },
    ];
    
    println!("Initial account states:");
    for account in &accounts {
        println!("  {}: ${:.2} @ {}% APY", account.id, account.balance, account.interest_rate * 100.0);
    }
    println!();
    
    // Create scheduler
    let scheduler = InterestAccrualScheduler::new(time.clone());
    
    // Spawn the scheduler task
    let accounts_clone = accounts.clone();
    let scheduler_handle = tokio::spawn(async move {
        let mut accounts = accounts_clone;
        scheduler.run_daily_accruals(&mut accounts).await;
        accounts
    });
    
    // Simulate 30 days of interest accrual
    println!("Simulating 30 days of daily interest accrual...\n");
    
    for day in 1..=30 {
        // Advance to next 2 AM
        let target = time.now().date_naive().succ_opt().unwrap()
            .and_hms_opt(2, 0, 0).unwrap().and_utc();
        control.set(target);
        
        // Give scheduler time to process
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        if day % 10 == 0 {
            println!("Day {} completed", day);
        }
    }
    
    // Advance one more day to ensure last accrual
    control.advance(Duration::days(1));
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    
    // Cancel the scheduler
    scheduler_handle.abort();
    
    // Calculate final results
    println!("\n=== Results after 30 days ===");
    
    let calculator = InterestCalculator::new(time.clone());
    
    // Manually calculate to show final state
    for (_i, initial_account) in accounts.iter().enumerate() {
        let (simple_interest, _) = calculator.calculate_interest(
            initial_account.balance,
            initial_account.interest_rate,
            "2024-01-01T00:00:00Z".parse().unwrap(),
        );
        
        let (final_amount, compound_interest, _) = calculator.calculate_compound_interest(
            initial_account.balance,
            initial_account.interest_rate,
            365, // Daily compounding
            "2024-01-01T00:00:00Z".parse().unwrap(),
        );
        
        println!("\nAccount {}:", initial_account.id);
        println!("  Initial balance: ${:.2}", initial_account.balance);
        println!("  Simple interest (30 days): ${:.2}", simple_interest);
        println!("  Compound interest (30 days): ${:.2}", compound_interest);
        println!("  Final balance (compound): ${:.2}", final_amount);
        println!("  Effective APY: {:.3}%", (compound_interest / initial_account.balance) * 365.0 / 30.0 * 100.0);
    }
    
    println!("\n=== Time Statistics ===");
    println!("Total wait calls: {}", control.wait_call_count());
    println!("Simulated {} days in milliseconds!", 30);
    println!("\nIn production, this would have taken 30 actual days!");
}