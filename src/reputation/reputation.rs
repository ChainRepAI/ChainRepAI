use serde::Serialize;
use uuid::Uuid;

use crate::{
    database::models::{RatingClassification, WalletMetrics},
    reputation::indicators::{
        DaysSinceLastBlock, PrioritizationFeesMetrics, TransactionFailureRate, TxPerHour,
        WalletBalance,
    },
    wallet::wallet::Wallet,
};

use super::indicators::{TransactionsWithNewWallets, WalletBalanceVolatility, WalletRewards};

#[derive(Serialize, Clone)]
pub struct Reputation {
    pub id: Uuid,
    pub penalties: Vec<ReputationPenalty>,
    pub rating_score: i32,
    pub rating_classification: RatingClassification,
    pub wallet_metrics: WalletMetrics,
}

impl Reputation {
    fn calc_rating_score(penalties: &[ReputationPenalty]) -> i32 {
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
        log::info!(
            "Initializing reputation creation for wallet with report id: {}",
            id
        );

        // calculate metrics/indicators
        let tx_per_hour = TxPerHour::calculate(&wallet.transaction_history);
        log::debug!("Computed transactions per hour: {:?}", tx_per_hour);

        let wallet_balance = WalletBalance(wallet.account_balance);
        log::debug!("Computed wallet balance: {:?}", wallet_balance);

        let days_since_last_block = DaysSinceLastBlock::calculate(&wallet.transaction_history)
            .unwrap_or_else(|| {
                log::warn!("Days since last block calculation failed, using default max value");
                DaysSinceLastBlock(u64::MAX)
            });
        log::debug!(
            "Computed days since last block: {:?}",
            days_since_last_block
        );

        let transaction_failure_rate =
            TransactionFailureRate::calculate(&wallet.transaction_history);
        log::debug!(
            "Computed transaction failure rate: {:?}",
            transaction_failure_rate
        );

        let prio_fee_metrics = PrioritizationFeesMetrics::calculate(&wallet.prioritization_fees);
        log::debug!(
            "Computed prioritization fees metrics: {:?}",
            prio_fee_metrics
        );

        let (fee_penalty_1, fee_penalty_2) = (&prio_fee_metrics).into();
        let penalties = vec![
            (&tx_per_hour).into(),
            (&wallet_balance).into(),
            (&days_since_last_block).into(),
            (&transaction_failure_rate).into(),
            fee_penalty_1,
            fee_penalty_2,
        ];

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
    pub severity: PenaltySeverity,
    pub reasoning: Vec<String>,
}

impl From<&WalletRewards> for ReputationPenalty {
    fn from(wallet_rewards: &WalletRewards) -> Self {
        let (severity, mut reasoning) = match wallet_rewards.0 {
            b if b < 1 => (
                PenaltySeverity::High,
                vec!["Wallet rewards < 1".to_string()],
            ),
            b if b < 5 => (
                PenaltySeverity::Medium,
                vec!["Balance between 1 and 5".to_string()],
            ),
            b if b < 20 => (
                PenaltySeverity::Low,
                vec!["Balance between 5 and 20".to_string()],
            ),
            _ => (
                PenaltySeverity::None,
                vec!["Balance >= 20 Solana".to_string()],
            ),
        };
        reasoning.push(format!("Wallet rewards: {:?}", wallet_rewards.0));
        Self {
            severity,
            reasoning,
        }
    }
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
            0 => (
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
            0 => (
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
}
