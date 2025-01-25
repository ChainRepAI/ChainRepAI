pub struct Reputation {
    pub items: Vec<ReputationItem>,
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