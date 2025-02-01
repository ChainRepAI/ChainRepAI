use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    openai_client::{openai_client::OpenAIClient, types::GeneratedCaseReportSections},
    reputation::reputation::Reputation,
    wallet::wallet::Wallet,
};

#[derive(Serialize, Deserialize)]
pub struct CaseReport {
    title: String,
    sections: GeneratedCaseReportSections,
}

impl CaseReport {
    pub async fn new(
        openai_client: &OpenAIClient,
        reputation: &Reputation,
        wallet: Wallet,
    ) -> Result<Self> {
        let title = format!(
            "Reputation Ratings Analysis of Wallet: {:?}",
            wallet.wallet_addr
        );
        let sections = openai_client.generate_case_report(&reputation).await?;

        Ok(CaseReport { title, sections })
    }
}
