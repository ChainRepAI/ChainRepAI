use anyhow::Result;
use diesel::{
    dsl::insert_into,
    query_dsl::methods::{FilterDsl, SelectDsl},
    Connection, ExpressionMethods, PgConnection, RunQueryDsl,
};
use uuid::Uuid;

use super::{
    models::{RatingClassification, WalletReport},
    schema::wallet_report,
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
}
