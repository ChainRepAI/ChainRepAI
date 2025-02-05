use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use solana_client::rpc_response::{
    RpcConfirmedTransactionStatusWithSignature, RpcPrioritizationFee,
};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use uuid::Uuid;

use crate::{
    database::models::{RatingClassification, WalletMetrics},
    wallet::wallet::Wallet,
};

pub struct TransactionsWithNewWallets(f64);

impl TransactionsWithNewWallets {
    pub fn calculate(
        confirmed_transactions: Vec<EncodedConfirmedTransactionWithStatusMeta>,
    ) -> Self {
        let total = confirmed_transactions.len();

        // Count transactions that include at least one account that appears to be newly funded.
        let count_new_wallets: usize = confirmed_transactions
            .iter()
            .filter(|tx| {
                if let Some(meta) = &tx.transaction.meta {
                    // Here we assume that meta includes pre_balances and post_balances as vectors of u64.
                    // A new wallet is assumed if any account's pre_balance is 0 and post_balance > 0.
                    meta.pre_balances
                        .iter()
                        .zip(meta.post_balances.iter())
                        .any(|(&pre, &post)| pre == 0 && post > 0)
                } else {
                    false
                }
            })
            .count();

        // Calculate the percentage; if there are no transactions, result defaults to 0.0.
        let percentage = if total > 0 {
            (count_new_wallets as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self(percentage)
    }
}

pub struct WalletBalanceVolatility(f64);

impl WalletBalanceVolatility {
    pub fn calculate(
        confirmed_transactions: Vec<EncodedConfirmedTransactionWithStatusMeta>,
    ) -> Self {
        let balances: Vec<f64> = confirmed_transactions
            .iter()
            .filter_map(|tx| {
                // Check if metadata exists and contains at least one balance in post_balances.
                tx.transaction
                    .meta
                    .as_ref()
                    .and_then(|meta| meta.post_balances.get(0))
                    .map(|balance| *balance as f64)
            })
            .collect();

        // if there are no valid balance entries, return zero volatility
        if balances.is_empty() {
            return Self(0.0);
        }

        // Calculate mean of the balances
        let sum: f64 = balances.iter().sum();
        let count = balances.len() as f64;
        let mean = sum / count;

        // Compute the variance: average of squared differences from the mean
        let variance_sum: f64 = balances
            .iter()
            .map(|balance| {
                let diff = balance - mean;
                diff * diff
            })
            .sum();
        let variance = variance_sum / count;

        // Volatility is the standard deviation
        let volatility = variance.sqrt();

        Self(volatility)
    }
}

#[derive(Debug)]
pub struct WalletBalance(u64);

#[derive(Debug)]
pub struct TxPerHour(i64);

impl TxPerHour {
    /// Calculates transaction volume over last 1000 transactions
    pub fn calculate(
        transaction_history: &Vec<RpcConfirmedTransactionStatusWithSignature>,
    ) -> Self {
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

#[derive(Debug)]
pub struct DaysSinceLastBlock(u64);

impl DaysSinceLastBlock {
    pub fn calculate(
        transaction_history: &Vec<RpcConfirmedTransactionStatusWithSignature>,
    ) -> Option<Self> {
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct TransactionFailureRate(f64);

impl TransactionFailureRate {
    pub fn calculate(
        transaction_history: &Vec<RpcConfirmedTransactionStatusWithSignature>,
    ) -> Self {
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
    pub id: Uuid,
    pub penalties: Vec<ReputationPenalty>,
    pub rating_score: i32,
    pub rating_classification: RatingClassification,
    pub wallet_metrics: WalletMetrics,
}

impl Reputation {
    fn calc_rating_score(penalties: &Vec<ReputationPenalty>) -> i32 {
        penalties.iter().fold(1000, |score, penalty| {
            score
                - match penalty.severity {
                    PenaltySeverity::High => 250,
                    PenaltySeverity::Medium => 150,
                    PenaltySeverity::Low => 50,
                    PenaltySeverity::None => 0,
                }
        })
    }

    pub fn new_from_wallet(wallet: &Wallet, id: Uuid) -> Self {
        log::info!("Initializing reputation creation for wallet with report id: {}", id);

        // calculate metrics/indicators
        let tx_per_hour = TxPerHour::calculate(&wallet.transaction_history);
        log::debug!("Computed transactions per hour: {:?}", tx_per_hour);

        let wallet_balance = WalletBalance(wallet.account_balance);
        log::debug!("Computed wallet balance: {:?}", wallet_balance);

        let days_since_last_block = DaysSinceLastBlock::calculate(&wallet.transaction_history)
            .unwrap_or_else(|| {
                log::warn!("Days since last block calculation failed, using default max value");
                DaysSinceLastBlock(std::u64::MAX)
            });
        log::debug!("Computed days since last block: {:?}", days_since_last_block);

        let transaction_failure_rate = TransactionFailureRate::calculate(&wallet.transaction_history);
        log::debug!("Computed transaction failure rate: {:?}", transaction_failure_rate);

        let prio_fee_metrics = PrioritizationFeesMetrics::calculate(&wallet.prioritization_fees);
        log::debug!("Computed prioritization fees metrics: {:?}", prio_fee_metrics);

        let mut penalties: Vec<ReputationPenalty> = vec![];
        // add penalties
        penalties.push((&tx_per_hour).into());
        penalties.push((&wallet_balance).into());
        penalties.push((&days_since_last_block).into());
        penalties.push((&transaction_failure_rate).into());
        let (fee_penalty_1, fee_penalty_2) = (&prio_fee_metrics).into();
        penalties.extend([fee_penalty_1, fee_penalty_2]);

        log::info!("Penalties calculated: {:?}", penalties);
        let rating_score = Self::calc_rating_score(&penalties);
        log::info!("Calculated rating score: {:?}", rating_score);

        Self {
            id,
            penalties,
            rating_score,
            rating_classification: rating_score.into(),
            wallet_metrics: WalletMetrics {
                wallet_report_id: id,
                transaction_failure_rate: transaction_failure_rate.0,
                avg_prio_fee: prio_fee_metrics.avg_fee,
                prio_fee_std_devi: prio_fee_metrics.std_deviation,
                days_since_last_block: days_since_last_block.0 as i64,
                tx_per_hour: tx_per_hour.0,
                wallet_balance: wallet_balance.0 as i64,
            },
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

#[derive(Serialize, Clone, Debug)]
pub struct ReputationPenalty {
    severity: PenaltySeverity,
    reasoning: Vec<String>,
}

impl From<&WalletBalance> for ReputationPenalty {
    fn from(balance: &WalletBalance) -> Self {
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
impl From<&TxPerHour> for ReputationPenalty {
    fn from(tx_per_hour: &TxPerHour) -> Self {
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

impl From<&DaysSinceLastBlock> for ReputationPenalty {
    fn from(days: &DaysSinceLastBlock) -> Self {
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

impl From<&TransactionFailureRate> for ReputationPenalty {
    fn from(failure_rate: &TransactionFailureRate) -> Self {
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

impl From<&PrioritizationFeesMetrics> for (ReputationPenalty, ReputationPenalty) {
    fn from(pfm: &PrioritizationFeesMetrics) -> Self {
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

impl From<&WalletBalanceVolatility> for ReputationPenalty {
    fn from(balance_volatility: &WalletBalanceVolatility) -> Self {
        let (severity, mut reasoning) = match balance_volatility.0 {
            f if f > 25.0 => (
                PenaltySeverity::High,
                vec!["Very high standard deviation in balance volatility".to_string()],
            ),
            f if f > 10.0 => (
                PenaltySeverity::Medium,
                vec!["Medium standard deviation in balance volatility".to_string()],
            ),
            f if f > 5.0 => (
                PenaltySeverity::Low,
                vec!["Low standard deviation in balance volatility".to_string()],
            ),
            _ => (
                PenaltySeverity::None,
                vec!["Very low standard deviation in balance volatility".to_string()],
            ),
        };
        reasoning.push(format!(
            "Balance volatility standard deviation: {:?}",
            balance_volatility.0
        ));

        Self {
            severity,
            reasoning,
        }
    }
}

impl From<&TransactionsWithNewWallets> for ReputationPenalty {
    fn from(transactions_with_new_wallets: &TransactionsWithNewWallets) -> Self {
        let (severity, mut reasoning) = match transactions_with_new_wallets.0 {
            p if p > 30.0 => (
                PenaltySeverity::High,
                vec!["Very high % of transactions with new wallets".to_string()],
            ),
            p if p > 20.0 => (
                PenaltySeverity::High,
                vec!["Medium/high % of transactions with new wallets".to_string()],
            ),
            p if p > 10.0 => (
                PenaltySeverity::High,
                vec!["Low % of transactions with new wallets".to_string()],
            ),
            _ => (
                PenaltySeverity::None,
                vec!["Very low % of transactions with new wallets".to_string()],
            ),
        };
        reasoning.push(format!(
            "% Transactions with new wallets: {:?}",
            transactions_with_new_wallets.0
        ));
        Self {
            severity,
            reasoning,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;

    fn create_mock_transaction(
        block_time: Option<i64>,
        has_error: bool,
    ) -> RpcConfirmedTransactionStatusWithSignature {
        RpcConfirmedTransactionStatusWithSignature {
            signature: String::new(),
            slot: 0,
            err: if has_error {
                Some(solana_sdk::transaction::TransactionError::AccountBorrowOutstanding)
            } else {
                None
            },
            memo: None,
            block_time,
            confirmation_status: None,
        }
    }

    #[test]
    fn test_wallet_balance_penalties() {
        let test_cases = vec![
            (500_000_000, PenaltySeverity::High),     // 0.5 SOL
            (5_000_000_000, PenaltySeverity::Medium), // 5 SOL
            (50_000_000_000, PenaltySeverity::Low),   // 50 SOL
            (150_000_000_000, PenaltySeverity::None), // 150 SOL
        ];

        for (balance, expected_severity) in test_cases {
            let penalty: ReputationPenalty = (&WalletBalance(balance)).into();
            assert_eq!(
                std::mem::discriminant(&penalty.severity),
                std::mem::discriminant(&expected_severity),
                "Balance {} should have {:?} severity",
                balance,
                expected_severity
            );
        }
    }

    #[test]
    fn test_tx_per_hour_calculation() {
        // Test case with 10 transactions over 2 hours
        let transactions: Vec<RpcConfirmedTransactionStatusWithSignature> = (0..10)
            .map(|i| create_mock_transaction(Some(1000 + i * 720), false))
            .collect();
        let tx_per_hour = TxPerHour::calculate(&transactions);
        assert_eq!(tx_per_hour.0, 10);
    }

    #[test]
    fn test_tx_per_hour_penalties() {
        let test_cases = vec![
            (0, PenaltySeverity::High),  // No transactions
            (3, PenaltySeverity::Low),   // Low volume
            (15, PenaltySeverity::None), // Good volume
            (30, PenaltySeverity::High), // Too high volume
        ];

        for (tx_per_hour, expected_severity) in test_cases {
            let penalty: ReputationPenalty = (&TxPerHour(tx_per_hour)).into();
            assert_eq!(
                std::mem::discriminant(&penalty.severity),
                std::mem::discriminant(&expected_severity),
                "TX per hour {} should have {:?} severity",
                tx_per_hour,
                expected_severity
            );
        }
    }

    #[test]
    fn test_days_since_last_block() {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let test_cases = vec![
            (current_time, PenaltySeverity::None),                // Today
            (current_time - 3 * 86400, PenaltySeverity::Low),     // 3 days ago
            (current_time - 15 * 86400, PenaltySeverity::Medium), // 15 days ago
            (current_time - 35 * 86400, PenaltySeverity::High),   // 35 days ago
        ];

        for (block_time, expected_severity) in test_cases {
            let transactions = vec![create_mock_transaction(Some(block_time as i64), false)];
            let days = DaysSinceLastBlock::calculate(&transactions).unwrap();
            let penalty: ReputationPenalty = (&days).into();
            assert_eq!(
                std::mem::discriminant(&penalty.severity),
                std::mem::discriminant(&expected_severity),
            );
        }
    }

    #[test]
    fn test_transaction_failure_rate() {
        let test_cases = vec![
            (vec![], 0.0),                     // Empty history
            (vec![false], 0.0),                // One successful transaction
            (vec![true], 100.0),               // One failed transaction
            (vec![false, false, true], 33.33), // Mixed case
        ];

        for (failure_pattern, expected_rate) in test_cases {
            let transactions: Vec<RpcConfirmedTransactionStatusWithSignature> = failure_pattern
                .into_iter()
                .map(|failed| create_mock_transaction(Some(1000), failed))
                .collect();

            let failure_rate = TransactionFailureRate::calculate(&transactions);
            assert!((failure_rate.0 - expected_rate).abs() < 0.01);
        }
    }

    #[test]
    fn test_prioritization_fees_metrics() {
        let fees = vec![
            RpcPrioritizationFee {
                slot: 0,
                prioritization_fee: 1,
            },
            RpcPrioritizationFee {
                slot: 0,
                prioritization_fee: 2,
            },
            RpcPrioritizationFee {
                slot: 0,
                prioritization_fee: 3,
            },
            RpcPrioritizationFee {
                slot: 0,
                prioritization_fee: 4,
            },
            RpcPrioritizationFee {
                slot: 0,
                prioritization_fee: 5,
            },
        ];
        let metrics = PrioritizationFeesMetrics::calculate(&fees);

        // Test average calculation
        assert!((metrics.avg_fee - 3.0).abs() < 0.001);

        // Test standard deviation calculation
        let expected_std_dev = (10.0f64).sqrt();
        assert!((metrics.std_deviation - expected_std_dev).abs() < 2.0);
    }
}
