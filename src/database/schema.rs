// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "rating_classification"))]
    pub struct RatingClassification;
}

diesel::table! {
    wallet_metrics (wallet_report_id) {
        wallet_report_id -> Uuid,
        transaction_failure_rate -> Float8,
        avg_prio_fee -> Float8,
        prio_fee_std_devi -> Float8,
        days_since_last_block -> Int8,
        tx_per_hour -> Int8,
        wallet_balance -> Int8,
    }
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
        wallet_addr -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        api_key -> Text,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(wallet_metrics, wallet_report,);
