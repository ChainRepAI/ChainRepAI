pub struct Reputation {
    pub items: Vec<ReputationItem>,
}
impl Reputation {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

enum ReputationLevel {
    High,
    Medium,
    Low,
    None,
}
struct ReputationItem {
    level: ReputationLevel,
    reasoning: Vec<String>,
}
impl From<&u64> for ReputationItem {
    fn from(bal: &u64) -> Self {
        let (level, reasoning) = match *bal {
            b if b < 1 => (
                ReputationLevel::None,
                vec!["Balance < 1 Solana".to_string()],
            ),
            b if b < 10 => (
                ReputationLevel::Medium,
                vec!["Balance between 1 and 10 Solana".to_string()],
            ),
            b if b < 100 => (
                ReputationLevel::Medium,
                vec!["Balance between 10 and 100 Solana".to_string()],
            ),
            _ => (
                ReputationLevel::High,
                vec!["Balance >= 100 Solana".to_string()],
            ),
        };

        Self { level, reasoning }
    }
}

/// Good transaction volume indicates high reputation
/// Too high volume indicates automation and hence, lower reputation
/// No volume indicates no reputation
impl From<&Vec<RpcConfirmedTransactionStatusWithSignature>> for ReputationItem {
    fn from(transaction_history: &Vec<RpcConfirmedTransactionStatusWithSignature>) -> Self {
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
        let tx_per_hour = transaction_history.len() as i64 / num_hours;

        let (level, reasoning) = match tx_per_hour {
            v if v == 0 => (
                ReputationLevel::None,
                vec!["No transaction volume".to_string()],
            ),
            v if v < 10 => (
                ReputationLevel::Medium,
                vec!["Low transaction volume".to_string()],
            ),
            v if v < 100 => (
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

impl From<&RpcConfirmedTransactionStatusWithSignature> for ReputationItem {
    fn from(first_transaction: &RpcConfirmedTransactionStatusWithSignature) -> Self {
        let days_wallet_active = match first_transaction.block_time {
            Some(tx_time) => {
                let timestamp_time = UNIX_EPOCH + Duration::from_secs(tx_time as u64);
                let now = SystemTime::now();

                now.duration_since(timestamp_time)
                    .expect("Invalid timestamp")
                    .as_secs()
                    / (24 * 3600)
            }
            None => 0,
        };
        let (level, reasoning) = match days_wallet_active {
            v if v == 0 => (ReputationLevel::None, vec!["Fresh wallet".to_string()]),
            v if v < 7 => (
                ReputationLevel::Low,
                vec!["Wallet less than a week old".to_string()],
            ),
            v if v < 30 => (
                ReputationLevel::Medium,
                vec!["Wallet less than a month old".to_string()],
            ),
            _ => (
                ReputationLevel::High,
                vec!["Wallet older than a month".to_string()],
            ),
        };
        Self { level, reasoning }
    }
}
