use anyhow::Result;
use diesel::{Connection, PgConnection};

pub struct Database {
    conn: PgConnection,
}

impl Database {
    pub fn connect() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let conn = PgConnection::establish(&database_url)?;
        Ok(Self { conn })
    }
}
