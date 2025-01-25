pub struct Reputation {
    items: Vec<ReputationItem>,
}

struct ReputationItem {
    score: u64,
    reasoning: Vec<String>,
}