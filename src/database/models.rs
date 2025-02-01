use chrono::NaiveDateTime;
use diesel::expression::AsExpression;
use diesel::prelude::{Insertable, Queryable};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, AsExpression, Serialize)]
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

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = crate::database::schema::wallet_report)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct WalletReport {
    id: Uuid,
    rating_classification: RatingClassification,
    rating_score: i32,
    case_report: serde_json::Value,
    report_creation_date: NaiveDateTime,
}
