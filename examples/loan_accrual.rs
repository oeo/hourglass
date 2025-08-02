use hourglass::{SafeTimeProvider, TimeSource};
use chrono::{DateTime, Duration, Utc, Datelike};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct LoanTerms {
    annual_rate: f64,
    accrual_interval: AccrualInterval,
    accrual_cycle_interval: AccrualCycleInterval,
    one_time_fee_rate: f64,
    _duration_months: u32,
    interest_due_days: i64,
    overdue_days: i64,
    liquidation_days: i64,
}

#[derive(Debug, Clone)]
enum AccrualInterval {
    EndOfDay,
    _EndOfMonth,
}

#[derive(Debug, Clone)]
enum AccrualCycleInterval {
    EndOfMonth,
    _EndOfWeek,
}

#[derive(Debug)]
struct Loan {
    id: String,
    facility: f64,
    disbursed_at: DateTime<Utc>,
    terms: LoanTerms,
    accrued_interest: f64,
    paid_interest: f64,
    last_accrual: DateTime<Utc>,
    last_cycle_close: DateTime<Utc>,
}

impl Loan {
    fn new(id: String, facility: f64, terms: LoanTerms, disbursed_at: DateTime<Utc>) -> Self {
        let one_time_fee = facility * terms.one_time_fee_rate / 100.0;
        Self {
            id,
            facility,
            disbursed_at,
            terms,
            accrued_interest: one_time_fee,
            paid_interest: 0.0,
            last_accrual: disbursed_at,
            last_cycle_close: disbursed_at,
        }
    }
    
    fn daily_rate(&self) -> f64 {
        self.terms.annual_rate / 365.0 / 100.0
    }
    
    fn outstanding_principal(&self) -> f64 {
        self.facility
    }
    
    fn is_overdue(&self, now: DateTime<Utc>) -> bool {
        let due_date = self.last_cycle_close + Duration::days(self.terms.interest_due_days);
        now > due_date + Duration::days(self.terms.overdue_days)
    }
    
    fn should_liquidate(&self, now: DateTime<Utc>) -> bool {
        let due_date = self.last_cycle_close + Duration::days(self.terms.interest_due_days);
        now > due_date + Duration::days(self.terms.liquidation_days)
    }
}

struct LoanAccrualEngine {
    time_provider: SafeTimeProvider,
    loans: HashMap<String, Loan>,
}

impl LoanAccrualEngine {
    fn new(time_provider: SafeTimeProvider) -> Self {
        Self {
            time_provider,
            loans: HashMap::new(),
        }
    }
    
    fn add_loan(&mut self, loan: Loan) {
        self.loans.insert(loan.id.clone(), loan);
    }
    
    async fn run_daily_accruals(&mut self) {
        let now = self.time_provider.now();
        println!("Running daily accruals at: {}", now);
        
        for loan in self.loans.values_mut() {
            // Only accrue if it's end of day
            if matches!(loan.terms.accrual_interval, AccrualInterval::EndOfDay) {
                let days_since_last = (now - loan.last_accrual).num_days();
                if days_since_last >= 1 {
                    let interest = loan.outstanding_principal() * loan.daily_rate() * days_since_last as f64;
                    loan.accrued_interest += interest;
                    loan.last_accrual = now;
                    
                    println!("  Loan {}: Accrued ${:.2} interest (total: ${:.2})", 
                        loan.id, interest, loan.accrued_interest);
                }
            }
        }
    }
    
    async fn run_monthly_cycle_close(&mut self) {
        let now = self.time_provider.now();
        
        for loan in self.loans.values_mut() {
            if matches!(loan.terms.accrual_cycle_interval, AccrualCycleInterval::EndOfMonth) {
                // Check if we're at month end
                let tomorrow = now + Duration::days(1);
                if now.month() != tomorrow.month() {
                    println!("  Loan {}: Month-end cycle close", loan.id);
                    println!("    Interest due: ${:.2}", loan.accrued_interest - loan.paid_interest);
                    loan.last_cycle_close = now;
                    
                    // Check status
                    if loan.is_overdue(now) {
                        println!("    Status: OVERDUE");
                    }
                    if loan.should_liquidate(now) {
                        println!("    Status: LIQUIDATION");
                    }
                }
            }
        }
    }
    
    async fn simulate_until(&mut self, end_date: DateTime<Utc>) {
        while self.time_provider.now() < end_date {
            self.run_daily_accruals().await;
            self.run_monthly_cycle_close().await;
            self.time_provider.wait(Duration::days(1)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Loan Accrual Simulation ===\n");
    
    // Test mode for simulation
    let time = SafeTimeProvider::new(
        TimeSource::Test("2024-01-15T00:00:00Z".parse().unwrap())
    );
    let control = time.test_control().expect("Should be in test mode");
    
    // Create loan terms matching the example
    let terms = LoanTerms {
        annual_rate: 12.0,
        accrual_interval: AccrualInterval::EndOfDay,
        accrual_cycle_interval: AccrualCycleInterval::EndOfMonth,
        one_time_fee_rate: 5.0,
        _duration_months: 3,
        interest_due_days: 0,
        overdue_days: 50,
        liquidation_days: 360,
    };
    
    // Create a loan
    let loan = Loan::new(
        "LOAN-001".to_string(),
        100_000.0,
        terms,
        time.now(),
    );
    
    println!("Loan created:");
    println!("  ID: {}", loan.id);
    println!("  Facility: ${:.2}", loan.facility);
    println!("  Annual Rate: {}%", loan.terms.annual_rate);
    println!("  One-time Fee: ${:.2} ({}%)", loan.accrued_interest, loan.terms.one_time_fee_rate);
    println!("  Disbursed: {}\n", loan.disbursed_at);
    
    // Create engine and add loan
    let mut engine = LoanAccrualEngine::new(time.clone());
    engine.add_loan(loan);
    
    // Simulate 4 months
    let end_date = time.now() + Duration::days(120);
    engine.simulate_until(end_date).await;
    
    // Show final state
    println!("\n=== Final State ===");
    for loan in engine.loans.values() {
        println!("Loan {}: Total accrued interest: ${:.2}", loan.id, loan.accrued_interest);
        
        let days_elapsed = (time.now() - loan.disbursed_at).num_days();
        let expected_interest = 5_000.0 + (100_000.0 * 0.12 / 365.0 * days_elapsed as f64);
        println!("Expected interest: ${:.2}", expected_interest);
        
        // Check overdue status
        if loan.is_overdue(time.now()) {
            println!("Status: OVERDUE (more than {} days past due)", loan.terms.overdue_days);
        }
    }
    
    println!("\n=== Time Statistics ===");
    println!("Total days simulated: {}", control.wait_call_count());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_daily_accruals() {
        let time = SafeTimeProvider::new(
            TimeSource::Test("2024-01-01T00:00:00Z".parse().unwrap())
        );
        let control = time.test_control().unwrap();
        
        let terms = LoanTerms {
            annual_rate: 12.0,
            accrual_interval: AccrualInterval::EndOfDay,
            accrual_cycle_interval: AccrualCycleInterval::EndOfMonth,
            one_time_fee_rate: 5.0,
            _duration_months: 3,
            interest_due_days: 0,
            overdue_days: 50,
            liquidation_days: 360,
        };
        
        let loan = Loan::new("TEST-001".to_string(), 100_000.0, terms, time.now());
        let mut engine = LoanAccrualEngine::new(time.clone());
        engine.add_loan(loan);
        
        // Advance 30 days
        control.advance(Duration::days(30));
        engine.run_daily_accruals().await;
        
        let loan = &engine.loans["TEST-001"];
        let expected = 5_000.0 + (100_000.0 * 0.12 / 365.0 * 30.0);
        assert!((loan.accrued_interest - expected).abs() < 0.01);
    }
    
    #[tokio::test]
    async fn test_overdue_detection() {
        let time = SafeTimeProvider::new(
            TimeSource::Test("2024-01-31T00:00:00Z".parse().unwrap())
        );
        let control = time.test_control().unwrap();
        
        let terms = LoanTerms {
            annual_rate: 12.0,
            accrual_interval: AccrualInterval::EndOfDay,
            accrual_cycle_interval: AccrualCycleInterval::EndOfMonth,
            one_time_fee_rate: 0.0,
            _duration_months: 3,
            interest_due_days: 0,
            overdue_days: 50,
            liquidation_days: 360,
        };
        
        let loan = Loan::new("TEST-002".to_string(), 100_000.0, terms, time.now());
        
        // Not overdue initially
        assert!(!loan.is_overdue(time.now()));
        
        // Still not overdue after 50 days
        control.advance(Duration::days(50));
        assert!(!loan.is_overdue(time.now()));
        
        // Overdue after 51 days
        control.advance(Duration::days(1));
        assert!(loan.is_overdue(time.now()));
    }
}