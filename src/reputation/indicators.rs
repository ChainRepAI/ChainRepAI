use std::time::{SystemTime, UNIX_EPOCH};

use solana_client::rpc_response::{
    RpcConfirmedTransactionStatusWithSignature, RpcPrioritizationFee,
};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

pub struct WalletRewards(pub i64);

impl WalletRewards {
    pub fn calculate(
        confirmed_transactions: &Vec<EncodedConfirmedTransactionWithStatusMeta>,
    ) -> Self {
        let total_rewards = confirmed_transactions
            .iter()
            .filter_map(|tx| tx.transaction.meta.as_ref())
            .flat_map(|meta| meta.rewards.clone().unwrap_or_else(Vec::new))
            .map(|reward| reward.lamports)
            .sum();

        Self(total_rewards)
    }
}

pub struct TransactionsWithNewWallets(pub f64);

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

pub struct WalletBalanceVolatility(pub f64);

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
pub struct WalletBalance(pub u64);

#[derive(Debug)]
pub struct TxPerHour(pub i64);

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
pub struct DaysSinceLastBlock(pub u64);

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
    pub avg_fee: f64,
    pub std_deviation: f64,
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
pub struct TransactionFailureRate(pub f64);

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

#[cfg(test)]
mod tests {
    use crate::reputation::reputation::{PenaltySeverity, ReputationPenalty};

    use super::*;

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
    fn test_tx_per_hour_calculation() {
        // Test case with 10 transactions over 2 hours
        let transactions: Vec<RpcConfirmedTransactionStatusWithSignature> = (0..10)
            .map(|i| create_mock_transaction(Some(1000 + i * 720), false))
            .collect();
        let tx_per_hour = TxPerHour::calculate(&transactions);
        assert_eq!(tx_per_hour.0, 10);
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
