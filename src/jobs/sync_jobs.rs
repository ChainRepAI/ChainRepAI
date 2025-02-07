use crate::{
    case_report::case_report::CaseReport,
    database::{
        models::{RatingClassification, User, WalletMetrics, WalletReport},
        postgres::Database,
    },
};
use anyhow::Result;
use chrono::NaiveDateTime;
use uuid::Uuid;

pub fn get_recent_wallet_reports(days: i64) -> Result<Vec<WalletReport>> {
    let mut database = Database::connect()?;
    database.get_recent_wallet_reports(days)
}

pub fn get_wallet_reports_by_classification(
    report_classification: RatingClassification,
) -> Result<Vec<WalletReport>> {
    let mut database = Database::connect()?;
    database.get_wallet_reports_by_classification(report_classification)
}

pub fn get_wallet_reports(from_score: i32, to_score: i32) -> Result<Vec<WalletReport>> {
    let mut database = Database::connect()?;
    database.get_reports_between_scores(from_score, to_score)
}

pub fn delete_user(api_key: &str) -> Result<()> {
    let mut database = Database::connect()?;
    database.delete_user(api_key)
}

pub fn delete_report(wallet_report_id: Uuid) -> Result<()> {
    let mut database = Database::connect()?;
    database.delete_report(wallet_report_id)
}

pub fn create_user() -> Result<String> {
    let mut database = Database::connect()?;
    let user = User::new();
    let api_key = user.api_key.clone();
    database.insert_user(user)?;
    Ok(api_key)
}

pub fn get_wallet_report_metrics(report_id: Uuid) -> Result<WalletMetrics> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_metrics(report_id)?)
}

pub fn get_wallet_report_count(wallet_addr: String) -> Result<i64> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_count(wallet_addr))
}

pub fn get_wallet_report_creation_date(report_id: Uuid) -> Result<NaiveDateTime> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_creation_date(report_id)?)
}

pub fn get_wallet_report_case_report(report_id: Uuid) -> Result<CaseReport> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_case_report(report_id)?)
}

pub fn get_wallet_report_score(report_id: Uuid) -> Result<i32> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_score(report_id)?)
}

pub fn get_wallet_report_classification(report_id: Uuid) -> Result<RatingClassification> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_classification(report_id)?)
}

pub fn get_wallet_report(report_id: Uuid) -> Result<WalletReport> {
    let mut database = Database::connect()?;
    database.get_wallet_report(report_id)
}
