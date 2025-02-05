use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::{
    delete, dsl::insert_into, Connection, ExpressionMethods, OptionalExtension, PgConnection,
    QueryDsl, RunQueryDsl,
};
use serde_json::from_value;
use uuid::Uuid;

use crate::case_report::case_report::CaseReport;

use super::{
    models::{RatingClassification, User, WalletMetrics, WalletReport},
    schema::{users, wallet_metrics, wallet_report},
};

pub struct Database {
    conn: PgConnection,
}

impl Database {
    pub fn connect() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let conn = PgConnection::establish(&database_url)?;
        Ok(Self { conn })
    }

    pub fn insert_wallet_report(&mut self, wallet_report: WalletReport) -> Result<()> {
        insert_into(wallet_report::table)
            .values(wallet_report)
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn get_wallet_report(&mut self, wallet_report_id: Uuid) -> Result<WalletReport> {
        Ok(wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?)
    }

    pub fn get_wallet_report_classification(
        &mut self,
        wallet_report_id: Uuid,
    ) -> Result<RatingClassification> {
        Ok(wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?
            .rating_classification)
    }

    pub fn get_wallet_report_score(&mut self, wallet_report_id: Uuid) -> Result<i32> {
        Ok(wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?
            .rating_score)
    }

    pub fn get_wallet_report_case_report(&mut self, wallet_report_id: Uuid) -> Result<CaseReport> {
        let case_report = wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?
            .case_report;
        Ok(from_value(case_report)?)
    }

    pub fn get_wallet_report_creation_date(
        &mut self,
        wallet_report_id: Uuid,
    ) -> Result<NaiveDateTime> {
        Ok(wallet_report::table
            .filter(wallet_report::id.eq(wallet_report_id))
            .select(wallet_report::all_columns)
            .first::<WalletReport>(&mut self.conn)?
            .report_creation_date)
    }

    pub fn get_wallet_report_count(&mut self, wallet_addr: String) -> i64 {
        wallet_report::table
            .filter(wallet_report::wallet_addr.eq(wallet_addr))
            .count()
            .get_result::<i64>(&mut self.conn)
            .unwrap_or_else(|_| 0)
    }

    pub fn insert_wallet_metrics(&mut self, wallet_metrics: WalletMetrics) -> Result<()> {
        insert_into(wallet_metrics::table)
            .values(wallet_metrics)
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn get_wallet_metrics(&mut self, wallet_report_id: Uuid) -> Result<WalletMetrics> {
        Ok(wallet_metrics::table
            .filter(wallet_metrics::wallet_report_id.eq(wallet_report_id))
            .first(&mut self.conn)?)
    }

    pub fn insert_user(&mut self, user: User) -> Result<()> {
        insert_into(users::table)
            .values(user)
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn check_user_exists(&mut self, api_key: &str) -> Result<bool> {
        let user = users::table
            .filter(users::api_key.eq(api_key.to_string()))
            .select(users::all_columns)
            .first::<User>(&mut self.conn)
            .optional()?;
        Ok(user.is_some())
    }

    pub fn delete_report(&mut self, wallet_report_id: Uuid) -> Result<()> {
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
        Ok(())
    }

    pub fn delete_user(&mut self, api_key: &str) -> Result<()> {
        delete(users::table)
            .filter(users::api_key.eq(api_key))
            .execute(&mut self.conn)?;
        Ok(())
    }
}
