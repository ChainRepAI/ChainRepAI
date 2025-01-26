use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletion {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: ChatMessage,
    pub logprobs: Option<serde_json::Value>,
    pub finish_reason: String,
}
