use std::env;

use anyhow::{Error, Result};
use reqwest::Client;
use serde_json::{json, to_string};

use crate::{openai_client::types::Message, reputation::reputation::Reputation};

use super::types::{ChatCompletion, GeneratedCaseReportSections};

const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";
const PROMPT: &str = "Using the following reputational information about a solana wallet, fill in the required fields. Title should be a short text describing the document.";

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

    pub async fn generate_case_report(
        &self,
        reputation: &Reputation,
    ) -> Result<GeneratedCaseReportSections> {
        let openai_response = self
            .client
            .post(OPENAI_URL)
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": "gpt-4o-mini-2024-07-18",
                "messages": vec![
                    Message { role: "system".to_string(), content: PROMPT.to_string() },
                    Message { role: "user".to_string(), content: to_string(reputation).unwrap() }
                ],
                "response_format": {
                    "type": "json_schema",
                    "json_schema": {
                        "strict": true,
                        "name": "GeneratedCaseReportSectionsResponse",
                        "schema": 
                        {
                            "$schema": "http://json-schema.org/draft-07/schema#",
                            "type": "object",
                            "required": ["summary", "reputation_strengths", "reputation_challenges", "potential_downgrade_factors", "penalty_breakdown"],
                            "properties": {
                                "summary": {
                                    "type": "string"
                                },
                                "reputation_strengths": {
                                    "type": "array",
                                    "items": {
                                        "type": "string"
                                    }
                                },
                                "reputation_challenges": {
                                    "type": "array",
                                    "items": {
                                        "type": "string"
                                    }
                                },
                                "potential_downgrade_factors": {
                                    "type": "array",
                                    "items": {
                                        "type": "string"
                                    }
                                },
                                "penalty_breakdown": {
                                    "type": "array",
                                    "items": {
                                        "type": "string"
                                    }
                                }
                            },
                            "additionalProperties": false
                        }
                    },
                }
            }))
            .send()
            .await?;

        let openai_response = openai_response.error_for_status()?;
        let completion: ChatCompletion = openai_response.json().await?;
        let choice = completion
            .choices
            .first()
            .ok_or_else(|| Error::msg("No choices available"))?;
        let content = choice
            .message
            .content
            .clone()
            .ok_or_else(|| Error::msg("No content available in the message"))?;
        let case_report_sections: GeneratedCaseReportSections = serde_json::from_str(&content)?;
        Ok(case_report_sections)
    }
}
