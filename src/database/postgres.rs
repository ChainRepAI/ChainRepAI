use anyhow::Result;
use diesel::{dsl::insert_into, Connection, PgConnection, RunQueryDsl};

use super::{models::WalletReport, schema::wallet_report};

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
}
