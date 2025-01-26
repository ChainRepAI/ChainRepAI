use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;

use crate::wallet::wallet::Wallet;

pub struct WalletBalance(u64);

pub struct TxPerHour(i64);

pub struct DaysSinceLastBlock(u64);

pub struct Reputation {
    pub items: Vec<ReputationItem>,
}

impl Reputation {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn calc_reputation(&mut self, wallet: &Wallet) {

    /// Calculates transaction volume over last 1000 transactions
    pub fn transaction_volume(&self, wallet: &Wallet) -> TxPerHour {
        let transaction_history = &wallet.transaction_history;
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
        TxPerHour(transaction_history.len() as i64 / num_hours)
    }

    pub fn dormancy(&self, wallet: &Wallet) -> Option<DaysSinceLastBlock> {
        wallet
            .transaction_history
            .last()
            .and_then(|last_tx| last_tx.block_time)
            .map(|block_time| {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                let time_diff = current_time.saturating_sub(block_time as u64);
                DaysSinceLastBlock(time_diff / (60 * 60 * 24))
            })
    }
    }
}

#[derive(Serialize)]
pub enum ReputationLevel {
    High,
    Medium,
    Low,
    None,
}

#[derive(Serialize)]
pub struct ReputationItem {
    level: ReputationLevel,
    reasoning: Vec<String>,
}

impl From<&u64> for ReputationItem {
    fn from(bal: &u64) -> Self {
        let (level, mut reasoning) = match *bal {
            b if b < 001000000000 => (
                ReputationLevel::None,
                vec!["Balance < 1 Solana".to_string()],
            ),
            b if b < 010000000000 => (
                ReputationLevel::Medium,
                vec!["Balance between 1 and 10 Solana".to_string()],
            ),
            b if b < 100000000000 => (
                ReputationLevel::Medium,
                vec!["Balance between 10 and 100 Solana".to_string()],
            ),
            _ => (
                ReputationLevel::High,
                vec!["Balance >= 100 Solana".to_string()],
            ),
        };
        reasoning.push(format!("Solana balance: {:?}", bal));

        Self { level, reasoning }
    }
}

/// Good transaction volume indicates high reputation
/// Too high volume indicates automation and hence, lower reputation
/// No volume indicates no reputation
impl From<TxPerHour> for ReputationItem {
    fn from(tx_per_hour: TxPerHour) -> Self {
        let (level, reasoning) = match tx_per_hour.0 {
            v if v == 0 => (
                ReputationLevel::None,
                vec!["No transaction volume".to_string()],
            ),
            v if v < 5 => (
                ReputationLevel::Medium,
                vec!["Low to Medium transaction volume".to_string()],
            ),
            v if v < 25 => (
                ReputationLevel::High,
                vec!["Reasonable level of transaction volume".to_string()],
            ),
            _ => (
                ReputationLevel::Low,
                vec!["Transaction volume too high".to_string()],
            ),
        };
        Self { level, reasoning }
    }
}

impl From<DaysSinceLastBlock> for ReputationItem {
    fn from(days: DaysSinceLastBlock) -> Self {
        let (level, reasoning) = match days.0 {
            d if d == 0 => (
                ReputationLevel::High,
                vec!["Very recent activity".to_string()],
            ),
            d if d < 7 => (ReputationLevel::Medium, vec!["Recent activity".to_string()]),
            d if d < 30 => (
                ReputationLevel::Low,
                vec!["Semi-recent activity".to_string()],
            ),
            _ => (
                ReputationLevel::None,
                vec!["Too many days without activity indicates dormancy".to_string()],
            ),
        };
        Self { level, reasoning }
    }
}
