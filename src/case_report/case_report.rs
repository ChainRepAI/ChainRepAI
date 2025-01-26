use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    openai_client::{openai_client::OpenAIClient, types::GeneratedCaseReportSections},
    reputation::reputation::{RatingClassification, Reputation},
    wallet::wallet::Wallet,
};

#[derive(Serialize)]
pub struct CaseReport {
    title: String,
    rating_classification: RatingClassification,
    rating_score: i32,
    sections: GeneratedCaseReportSections,
    report_creation_date: DateTime<Utc>,
}
