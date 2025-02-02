use std::io::Write;

use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::{Insertable, Queryable};
use diesel::serialize::{self, IsNull, Output, ToSql};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::case_report::case_report::CaseReport;

#[derive(Debug, AsExpression, FromSqlRow, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = crate::database::schema::sql_types::RatingClassification)]
pub enum RatingClassification {
    AAA,
    AA,
    A,
    BBB,
    BB,
    B,
    CCC,
    CC,
    C,
}

impl ToSql<crate::database::schema::sql_types::RatingClassification, Pg> for RatingClassification {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            RatingClassification::AAA => out.write_all(b"aaa")?,
            RatingClassification::AA => out.write_all(b"aa")?,
            RatingClassification::A => out.write_all(b"a")?,
            RatingClassification::BBB => out.write_all(b"bbb")?,
            RatingClassification::BB => out.write_all(b"bb")?,
            RatingClassification::B => out.write_all(b"b")?,
            RatingClassification::CCC => out.write_all(b"ccc")?,
            RatingClassification::CC => out.write_all(b"cc")?,
            RatingClassification::C => out.write_all(b"c")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::database::schema::sql_types::RatingClassification, Pg>
    for RatingClassification
{
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"aaa" => Ok(RatingClassification::AAA),
            b"aa" => Ok(RatingClassification::AA),
            b"a" => Ok(RatingClassification::A),
            b"bbb" => Ok(RatingClassification::BBB),
            b"bb" => Ok(RatingClassification::BB),
            b"b" => Ok(RatingClassification::B),
            b"ccc" => Ok(RatingClassification::CCC),
            b"cc" => Ok(RatingClassification::CC),
            b"c" => Ok(RatingClassification::C),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<i32> for RatingClassification {
    fn from(rating_score: i32) -> Self {
        match rating_score {
            s if s < 200 => RatingClassification::C,
            s if s < 300 => RatingClassification::CC,
            s if s < 400 => RatingClassification::CCC,
            s if s < 500 => RatingClassification::B,
            s if s < 600 => RatingClassification::BB,
            s if s < 700 => RatingClassification::BBB,
            s if s < 800 => RatingClassification::A,
            s if s < 900 => RatingClassification::AA,
            _ => RatingClassification::AAA,
        }
    }
}

#[derive(Insertable, Queryable, Debug, Serialize)]
#[diesel(table_name = crate::database::schema::wallet_report)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WalletReport {
    id: Uuid,
    pub rating_classification: RatingClassification,
    pub rating_score: i32,
    pub case_report: serde_json::Value,
    pub report_creation_date: NaiveDateTime,
    wallet_addr: String,
}

impl WalletReport {
    pub fn new(
        id: Uuid,
        rating_classification: RatingClassification,
        rating_score: i32,
        case_report: CaseReport,
        wallet_addr: String,
    ) -> Result<Self> {
        Ok(Self {
            id,
            rating_classification,
            rating_score,
            case_report: serde_json::to_value(case_report)?,
            report_creation_date: Utc::now().naive_local(),
            wallet_addr,
        })
    }
}

#[derive(Insertable, Queryable, Debug, Serialize, Clone)]
#[diesel(table_name = crate::database::schema::wallet_metrics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WalletMetrics {
    pub wallet_report_id: Uuid,
    pub transaction_failure_rate: f64,
    pub avg_prio_fee: f64,
    pub prio_fee_std_devi: f64,
    pub days_since_last_block: i64,
    pub tx_per_hour: i64,
    pub wallet_balance: i64,
}
