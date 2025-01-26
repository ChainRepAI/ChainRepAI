use std::env;
use reqwest::Client;
use serde_json::{json, to_string};

pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

impl OpenAIClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: env::var("OPENAI_API_KEY").expect("OpenAI api key should be set"),
        }
    }
}
