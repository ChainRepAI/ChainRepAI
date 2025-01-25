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
