// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "rating_classification"))]
    pub struct RatingClassification;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RatingClassification;

    wallet_report (id) {
        id -> Uuid,
        rating_classification -> RatingClassification,
        rating_score -> Int4,
        case_report -> Jsonb,
        report_creation_date -> Timestamp,
    }
}
