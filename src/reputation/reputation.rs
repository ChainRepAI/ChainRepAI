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