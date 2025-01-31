use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable};
use diesel::sql_types::SqlType;
use diesel_derive_enum::DbEnum;
use uuid::Uuid;
use diesel::expression::AsExpression;

#[derive(Debug, AsExpression, DbEnum)]
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

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = crate::database::schema::wallet_report)]
#[diesel(check_for_backend(diesel::pg::Pg))]struct WalletReport {
    id: Uuid,
    rating_classification: RatingClassification,
    rating_score: i32,
    case_report: serde_json::Value,
    report_creation_date: NaiveDateTime,
}
