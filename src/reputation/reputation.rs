use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use solana_client::rpc_response::{RpcConfirmedTransactionStatusWithSignature, RpcPrioritizationFee};

use crate::{database::models::RatingClassification, wallet::wallet::Wallet};

pub struct WalletBalance(u64);

pub struct TxPerHour(i64);

impl TxPerHour {
    /// Calculates transaction volume over last 1000 transactions
    pub fn calculate(transaction_history: &Vec<RpcConfirmedTransactionStatusWithSignature>) -> Self {
        let num_hours = transaction_history
            .first()
            .and_then(|first_tx| first_tx.block_time)
            .zip(
                transaction_history
                    .last()
                    .and_then(|last_tx| last_tx.block_time),
            )
            .map(|(first_tx_time, last_tx_time)| (first_tx_time - last_tx_time).abs() / 3600)
            .unwrap_or(0);
        Self(transaction_history.len() as i64 / num_hours)
    }
}

pub struct DaysSinceLastBlock(u64);

impl DaysSinceLastBlock {
    pub fn calculate(transaction_history: &Vec<RpcConfirmedTransactionStatusWithSignature>) -> Option<Self> {
            transaction_history
            .last()
            .and_then(|last_tx| last_tx.block_time)
            .map(|block_time| {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                let time_diff = current_time.saturating_sub(block_time as u64);
                Self(time_diff / (60 * 60 * 24))
            })
    }
}

pub struct PrioritizationFeesMetrics {
    avg_fee: f64,
    std_deviation: f64,
}

impl PrioritizationFeesMetrics {
    pub fn calculate(prioritization_fees: &Vec<RpcPrioritizationFee>) -> Self {
        let fees: Vec<f64> = prioritization_fees
            .iter()
            .map(|fee| fee.prioritization_fee as f64)
            .collect();
        let avg_fee = fees.iter().sum::<f64>() / fees.len() as f64;

        let variance = fees
            .iter()
            .map(|fee| {
                let diff = fee - avg_fee;
                diff * diff
            })
            .sum::<f64>()
            / fees.len() as f64;

        let std_deviation = variance.sqrt();

        Self {
            avg_fee,
            std_deviation,
        }
    }
}

pub struct TransactionFailureRate(f64);

impl TransactionFailureRate {
    pub fn calculate(transaction_history: &Vec<RpcConfirmedTransactionStatusWithSignature>) -> Self {        
        if transaction_history.is_empty() {
            return Self(0.0);
        }

        let total_transactions = transaction_history.len() as f64;
        let failed_transactions = transaction_history
            .iter()
            .filter(|tx| tx.err.is_some())
            .count() as f64;

        let failure_rate = if total_transactions == 0.0 {
            0.0
        } else {
            (failed_transactions / total_transactions) * 100.0
        };

        Self(failure_rate)
    }
}

#[derive(Serialize, Clone)]
pub struct Reputation {
    pub penalties: Vec<ReputationPenalty>,
    pub rating_score: i32,
    pub rating_classification: RatingClassification,
}

impl Reputation {
    pub fn new_from_wallet(wallet: &Wallet) -> Self {
        let mut penalties: Vec<ReputationPenalty> = vec![];
        // add penalties
        penalties.push(TxPerHour::calculate(&wallet.transaction_history).into());
        penalties.push(WalletBalance(wallet.account_balance).into());
        penalties.push(
            DaysSinceLastBlock::calculate(&wallet.transaction_history)
                .unwrap_or_else(|| DaysSinceLastBlock(std::u64::MAX))
                .into(),
        );
        penalties.push(TransactionFailureRate::calculate(&wallet.transaction_history).into());
        let (fee_penalty_1, fee_penalty_2) = PrioritizationFeesMetrics::calculate(&wallet.prioritization_fees).into();
        penalties.extend([fee_penalty_1, fee_penalty_2]);

        let rating_score = penalties.iter().fold(1000, |score, penalty| {
            score
                - match penalty.severity {
                    PenaltySeverity::High => 250,
                    PenaltySeverity::Medium => 150,
                    PenaltySeverity::Low => 50,
                    PenaltySeverity::None => 0,
                }
        });

        Self {
            penalties,
            rating_score,
            rating_classification: rating_score.into(),
        }
    }
}

#[derive(Serialize, Clone, Copy, Debug)]
pub enum PenaltySeverity {
    High,
    Medium,
    Low,
    None,
}

#[derive(Serialize, Clone)]
pub struct ReputationPenalty {
    severity: PenaltySeverity,
    reasoning: Vec<String>,
}

impl From<WalletBalance> for ReputationPenalty {
    fn from(balance: WalletBalance) -> Self {
        let (severity, mut reasoning) = match balance.0 {
            b if b < 001000000000 => (
                PenaltySeverity::High,
                vec!["Balance < 1 Solana".to_string()],
            ),
            b if b < 010000000000 => (
                PenaltySeverity::Medium,
                vec!["Balance between 1 and 10 Solana".to_string()],
            ),
            b if b < 100000000000 => (
                PenaltySeverity::Low,
                vec!["Balance between 10 and 100 Solana".to_string()],
            ),
            _ => (
                PenaltySeverity::None,
                vec!["Balance >= 100 Solana".to_string()],
            ),
        };
        reasoning.push(format!("Solana balance: {:?}", balance.0 / 1_000_000_000));
        Self {
            severity,
            reasoning,
        }
    }
}

/// Good transaction volume indicates high reputation
/// Too high volume indicates automation and hence, lower reputation
/// No volume indicates no reputation
impl From<TxPerHour> for ReputationPenalty {
    fn from(tx_per_hour: TxPerHour) -> Self {
        let (severity, mut reasoning) = match tx_per_hour.0 {
            v if v == 0 => (
                PenaltySeverity::High,
                vec!["No transaction volume".to_string()],
            ),
            v if v < 5 => (
                PenaltySeverity::Low,
                vec!["Low to Medium transaction volume".to_string()],
            ),
            v if v < 25 => (
                PenaltySeverity::None,
                vec!["Reasonable level of transaction volume".to_string()],
            ),
            _ => (
                PenaltySeverity::High,
                vec!["Transaction volume too high".to_string()],
            ),
        };
        reasoning.push(format!("Transaction volumer per hour: {:?}", tx_per_hour.0));
        Self {
            severity,
            reasoning,
        }
    }
}

impl From<DaysSinceLastBlock> for ReputationPenalty {
    fn from(days: DaysSinceLastBlock) -> Self {
        let (severity, mut reasoning) = match days.0 {
            d if d == 0 => (
                PenaltySeverity::None,
                vec!["Recent activity in less than a day".to_string()],
            ),
            d if d < 7 => (
                PenaltySeverity::Low,
                vec!["Recent activity, less than a week ago".to_string()],
            ),
            d if d < 30 => (
                PenaltySeverity::Medium,
                vec!["Activity less than a month ago".to_string()],
            ),
            _ => (
                PenaltySeverity::High,
                vec!["No activity within a month".to_string()],
            ),
        };
        reasoning.push(format!("Days since last transactioon: {:?}", days.0));
        Self {
            severity,
            reasoning,
        }
    }
}

impl From<TransactionFailureRate> for ReputationPenalty {
    fn from(failure_rate: TransactionFailureRate) -> Self {
        let (severity, mut reasoning) = match failure_rate.0 {
            f if f > 10.0 => (
                PenaltySeverity::High,
                vec!["High transaction failure rate".to_string()],
            ),
            f if f > 5.0 => (
                PenaltySeverity::Medium,
                vec!["Moderate transaction failure rate".to_string()],
            ),
            f if f > 0.0 => (
                PenaltySeverity::Low,
                vec!["Low transaction failure rate".to_string()],
            ),
            _ => (
                PenaltySeverity::None,
                vec!["No transaction failures".to_string()],
            ),
        };
        reasoning.push(format!("Transaction failure rate: {:?}", failure_rate.0));
        Self {
            severity,
            reasoning,
        }
    }
}

impl From<PrioritizationFeesMetrics> for (ReputationPenalty, ReputationPenalty) {
    fn from(pfm: PrioritizationFeesMetrics) -> Self {
        let (avg_fee_severity, mut avg_fee_reasoning) = match pfm.avg_fee {
            f if f > 10.0 => (
                PenaltySeverity::None,
                vec!["High average prioritization fee".to_string()],
            ),
            f if f > 5.0 => (
                PenaltySeverity::None,
                vec!["Medium average prioritization fee".to_string()],
            ),
            f if f > 0.0 => (
                PenaltySeverity::None,
                vec!["Low average prioritization fee".to_string()],
            ),
            _ => (
                PenaltySeverity::High,
                vec!["No prioritization fee used".to_string()],
            ),
        };
        avg_fee_reasoning.push(format!("Average prioritization fee: {:?}", pfm.avg_fee));

        let (std_deviation_severity, mut std_deviation_reasoning) = match pfm.std_deviation {
            f if f > 50.0 => (
                PenaltySeverity::High,
                vec!["Very high standard deviation in prioritization fee".to_string()],
            ),
            f if f > 25.0 => (
                PenaltySeverity::Medium,
                vec!["Medium standard deviation in prioritization fee".to_string()],
            ),
            f if f > 5.0 => (
                PenaltySeverity::Low,
                vec!["Low standard deviation in prioritization fee".to_string()],
            ),
            _ => (
                PenaltySeverity::None,
                vec!["Very low standard deviation in prioritization fee".to_string()],
            ),
        };
        std_deviation_reasoning.push(format!(
            "Prioritization fee standard deviation: {:?}",
            pfm.std_deviation
        ));

        (
            ReputationPenalty {
                severity: avg_fee_severity,
                reasoning: avg_fee_reasoning,
            },
            ReputationPenalty {
                severity: std_deviation_severity,
                reasoning: std_deviation_reasoning,
            },
        )
    }
}
