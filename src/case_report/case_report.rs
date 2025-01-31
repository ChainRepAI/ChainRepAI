use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    database::models::RatingClassification, openai_client::{openai_client::OpenAIClient, types::GeneratedCaseReportSections}, reputation::reputation::Reputation, wallet::wallet::Wallet
};

#[derive(Serialize)]
pub struct CaseReport {
    title: String,
    rating_classification: RatingClassification,
    rating_score: i32,
    sections: GeneratedCaseReportSections,
    report_creation_date: DateTime<Utc>,
}

impl CaseReport {
    pub async fn new(
        openai_client: &OpenAIClient,
        reputation: Reputation,
        wallet: Wallet,
    ) -> Result<Self> {
        let title = format!(
            "Reputation Ratings Analysis of Wallet: {:?}",
            wallet.wallet_addr
        );
        let sections = openai_client.generate_case_report(&reputation).await?;

        Ok(CaseReport {
            title,
            rating_classification: reputation.rating_classification,
            rating_score: reputation.rating_score,
            sections,
            report_creation_date: Utc::now(),
        })
    }
}
