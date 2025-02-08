use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::{
    delete, dsl::insert_into, Connection, ExpressionMethods, OptionalExtension, PgConnection,
    QueryDsl, RunQueryDsl,
};
use log::{error, info};
use serde_json::from_value;
use uuid::Uuid;

use crate::case_report::case_report::CaseReport;

use super::{
    models::{
        KnownCreditedWallet, KnownDiscreditedWallet, RatingClassification, User, WalletMetrics,
        WalletReport,
    },
    schema::{
        known_credited_wallets, known_discredited_wallets, users, wallet_metrics, wallet_report,
    },
};

pub struct Database {
    conn: PgConnection,
}

impl Database {
    pub fn connect() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        info!(
            "Attempting to establish a connection to the database at {}",
            database_url
        );
        let conn = PgConnection::establish(&database_url)?;
        info!("Successfully connected to the database");
        Ok(Self { conn })
    }

    pub fn insert_wallet_report(&mut self, wallet_report: WalletReport) -> Result<()> {
        insert_into(wallet_report::table)
            .values(&wallet_report)
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn get_wallet_report(&mut self, wallet_report_id: Uuid) -> Result<WalletReport> {
        info!("Fetching wallet report with id: {}", wallet_report_id);
        let report = wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?;
        info!(
            "Successfully fetched wallet report with id: {}",
            wallet_report_id
        );
        Ok(report)
    }

    pub fn get_wallet_report_classification(
        &mut self,
        wallet_report_id: Uuid,
    ) -> Result<RatingClassification> {
        info!(
            "Fetching rating classification for wallet report id: {}",
            wallet_report_id
        );
        let classification = wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?
            .rating_classification;
        info!(
            "Successfully fetched rating classification for wallet report id: {}",
            wallet_report_id
        );
        Ok(classification)
    }

    pub fn get_wallet_report_score(&mut self, wallet_report_id: Uuid) -> Result<i32> {
        info!(
            "Fetching rating score for wallet report id: {}",
            wallet_report_id
        );
        let score = wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?
            .rating_score;
        info!(
            "Successfully fetched rating score for wallet report id: {}",
            wallet_report_id
        );
        Ok(score)
    }

    pub fn get_wallet_report_case_report(&mut self, wallet_report_id: Uuid) -> Result<CaseReport> {
        info!(
            "Fetching case report for wallet report id: {}",
            wallet_report_id
        );
        let case_report_value = wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?
            .case_report;
        let case_report: CaseReport = from_value(case_report_value)?;
        info!(
            "Successfully fetched case report for wallet report id: {}",
            wallet_report_id
        );
        Ok(case_report)
    }

    pub fn get_wallet_report_creation_date(
        &mut self,
        wallet_report_id: Uuid,
    ) -> Result<NaiveDateTime> {
        info!(
            "Fetching creation date for wallet report id: {}",
            wallet_report_id
        );
        let creation_date = wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?
            .report_creation_date;
        info!(
            "Successfully fetched creation date for wallet report id: {}",
            wallet_report_id
        );
        Ok(creation_date)
    }

    pub fn get_wallet_report_count(&mut self, wallet_addr: String) -> i64 {
        info!("Counting wallet reports for wallet_addr: {}", wallet_addr);
        let count = wallet_report::table
            .filter(wallet_report::wallet_addr.eq(wallet_addr.clone()))
            .count()
            .get_result::<i64>(&mut self.conn)
            .unwrap_or_else(|e| {
                error!(
                    "Failed to count wallet reports for address {}: {}",
                    wallet_addr, e
                );
                0
            });
        info!("Wallet report count for {}: {}", wallet_addr, count);
        count
    }

    pub fn insert_wallet_metrics(&mut self, wallet_metrics: WalletMetrics) -> Result<()> {
        info!(
            "Inserting wallet metrics for wallet_report_id: {}",
            wallet_metrics.wallet_report_id
        );
        insert_into(wallet_metrics::table)
            .values(&wallet_metrics)
            .execute(&mut self.conn)?;
        info!(
            "Successfully inserted wallet metrics for wallet_report_id: {}",
            wallet_metrics.wallet_report_id
        );
        Ok(())
    }

    pub fn get_wallet_metrics(&mut self, wallet_report_id: Uuid) -> Result<WalletMetrics> {
        info!(
            "Fetching wallet metrics for wallet_report_id: {}",
            wallet_report_id
        );
        let metrics = wallet_metrics::table
            .filter(wallet_metrics::wallet_report_id.eq(wallet_report_id))
            .first(&mut self.conn)?;
        info!(
            "Successfully fetched wallet metrics for wallet_report_id: {}",
            wallet_report_id
        );
        Ok(metrics)
    }

    pub fn insert_user(&mut self, user: User) -> Result<()> {
        info!("Inserting user with api_key: {}", user.api_key);
        insert_into(users::table)
            .values(&user)
            .execute(&mut self.conn)?;
        info!("Successfully inserted user with api_key: {}", user.api_key);
        Ok(())
    }

    pub fn check_user_exists(&mut self, api_key: &str) -> Result<bool> {
        info!("Checking existence of user with api_key: {}", api_key);
        let user = users::table
            .filter(users::api_key.eq(api_key.to_string()))
            .select(users::all_columns)
            .first::<User>(&mut self.conn)
            .optional()?;
        let exists = user.is_some();
        info!("User with api_key: {} exists: {}", api_key, exists);
        Ok(exists)
    }

    pub fn delete_report(&mut self, wallet_report_id: Uuid) -> Result<()> {
        info!("Deleting report for wallet_report_id: {}", wallet_report_id);
        self.conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                delete(wallet_metrics::table)
                    .filter(wallet_metrics::wallet_report_id.eq(wallet_report_id))
                    .execute(conn)?;

                delete(wallet_report::table)
                    .filter(wallet_report::id.eq(wallet_report_id))
                    .execute(conn)?;
                Ok(())
            })?;
        info!(
            "Successfully deleted report and associated metrics for wallet_report_id: {}",
            wallet_report_id
        );
        Ok(())
    }

    pub fn delete_user(&mut self, api_key: &str) -> Result<()> {
        info!("Deleting user with api_key: {}", api_key);
        delete(users::table)
            .filter(users::api_key.eq(api_key))
            .execute(&mut self.conn)?;
        info!("Successfully deleted user with api_key: {}", api_key);
        Ok(())
    }

    pub fn get_reports_between_scores(
        &mut self,
        from_score: i32,
        to_score: i32,
    ) -> Result<Vec<WalletReport>> {
        Ok(wallet_report::table
            .filter(wallet_report::rating_score.ge(from_score))
            .filter(wallet_report::rating_score.le(to_score))
            .get_results(&mut self.conn)?)
    }

    pub fn get_wallet_reports_by_classification(
        &mut self,
        classification: RatingClassification,
    ) -> Result<Vec<WalletReport>> {
        let reports = wallet_report::table
            .filter(wallet_report::rating_classification.eq(classification))
            .load::<WalletReport>(&mut self.conn)?;
        Ok(reports)
    }

    pub fn get_recent_wallet_reports(&mut self, days: i64) -> Result<Vec<WalletReport>> {
        let recent_date = chrono::Utc::now().naive_utc() - chrono::Duration::days(days);
        let reports = wallet_report::table
            .filter(wallet_report::report_creation_date.ge(recent_date))
            .load::<WalletReport>(&mut self.conn)?;
        info!(
            "Successfully fetched wallet reports from the last {} days",
            days
        );
        Ok(reports)
    }

    pub fn insert_discredited_wallet(
        &mut self,
        known_discredited_wallet: KnownDiscreditedWallet,
    ) -> Result<()> {
        insert_into(known_discredited_wallets::table)
            .values(known_discredited_wallet)
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn find_discredited_associates(
        &mut self,
        associated_wallets: Vec<String>,
    ) -> Result<Vec<KnownDiscreditedWallet>> {
        Ok(known_discredited_wallets::table
            .filter(known_discredited_wallets::wallet_addr.eq_any(associated_wallets))
            .select(known_discredited_wallets::all_columns)
            .get_results(&mut self.conn)?)
    }

    pub fn insert_credited_wallet(
        &mut self,
        known_credited_wallet: KnownCreditedWallet,
    ) -> Result<()> {
        insert_into(known_credited_wallets::table)
            .values(known_credited_wallet)
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn find_credited_associates(
        &mut self,
        associated_wallets: Vec<String>,
    ) -> Result<Vec<KnownCreditedWallet>> {
        Ok(known_credited_wallets::table
            .filter(known_credited_wallets::wallet_addr.eq_any(associated_wallets))
            .select(known_credited_wallets::all_columns)
            .get_results(&mut self.conn)?)
    }
}
