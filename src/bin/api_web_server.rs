use std::{collections::HashMap, ops::Deref};

use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use chrono::NaiveDateTime;
use dotenv::dotenv;
use log::{error, info};
use uuid::Uuid;
use SolAnalystAI::{
    case_report::case_report::CaseReport,
    database::{
        models::{RatingClassification, User, WalletMetrics, WalletReport},
        postgres::Database,
    },
    jobs::jobs::WalletReportJob,
    pulsar::pulsar::PulsarClient,
    worker::worker::WALLET_REPUTATION_TOPIC,
};

fn get_wallet_reports_by_classification(
    report_classification: RatingClassification,
) -> Result<Vec<WalletReport>> {
    let mut database = Database::connect()?;
    database.get_wallet_reports_by_classification(report_classification)
}

fn get_wallet_reports(from_score: i32, to_score: i32) -> Result<Vec<WalletReport>> {
    let mut database = Database::connect()?;
    database.get_reports_between_scores(from_score, to_score)
}

fn delete_user(api_key: &str) -> Result<()> {
    let mut database = Database::connect()?;
    database.delete_user(api_key)
}

fn delete_report(wallet_report_id: Uuid) -> Result<()> {
    let mut database = Database::connect()?;
    database.delete_report(wallet_report_id)
}

fn create_user() -> Result<String> {
    let mut database = Database::connect()?;
    let user = User::new();
    let api_key = user.api_key.clone();
    database.insert_user(user)?;
    Ok(api_key)
}

fn get_wallet_report_metrics(report_id: Uuid) -> Result<WalletMetrics> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_metrics(report_id)?)
}

fn get_wallet_report_count(wallet_addr: String) -> Result<i64> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_count(wallet_addr))
}

fn get_wallet_report_creation_date(report_id: Uuid) -> Result<NaiveDateTime> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_creation_date(report_id)?)
}

fn get_wallet_report_case_report(report_id: Uuid) -> Result<CaseReport> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_case_report(report_id)?)
}

fn get_wallet_report_score(report_id: Uuid) -> Result<i32> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_score(report_id)?)
}

fn get_wallet_report_classification(report_id: Uuid) -> Result<RatingClassification> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report_classification(report_id)?)
}

fn get_wallet_report(report_id: Uuid) -> Result<WalletReport> {
    let mut database = Database::connect()?;
    database.get_wallet_report(report_id)
}

#[get("/get_wallet_reports_by_classification/{report_classification}")]
async fn get_wallet_reports_by_classification_endpoint(
    report_classification: web::Path<RatingClassification>,
) -> impl Responder {
    match get_wallet_reports_by_classification(report_classification.to_owned()) {
        Ok(wallet_reports) => HttpResponse::Ok().json(wallet_reports),
        Err(_) => HttpResponse::InternalServerError().json("Unable to fetch wallet reports."),
    }
}

#[get("/get_wallets")]
async fn get_wallet_reports_endpoint(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let from_score = query
        .get("from_score")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    let to_score = query
        .get("to_score")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1000);

    match get_wallet_reports(from_score, to_score) {
        Ok(wallet_reports) => HttpResponse::Ok().json(wallet_reports),
        Err(_) => HttpResponse::InternalServerError()
            .json("Unable to fetch wallets between specified values"),
    }
}

#[delete("/delete_user/{api_key}")]
async fn delete_user_endpoint(api_key: web::Path<String>) -> impl Responder {
    info!("Received delete user request for API key: {}", api_key);
    match delete_user(api_key.as_str()) {
        Ok(_) => {
            info!("Successfully deleted user with API key: {}", api_key);
            HttpResponse::Ok().json("Successfully deleted user")
        }
        Err(err) => {
            error!("Failed to delete user with API key {}: {:?}", api_key, err);
            HttpResponse::InternalServerError().json("Unable to process request")
        }
    }
}

#[delete("/delete_wallet_report/{report_id}")]
async fn delete_wallet_report_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    info!(
        "Received delete wallet report request for report ID: {}",
        report_id
    );
    match delete_report(*report_id) {
        Ok(_) => {
            info!("Successfully deleted wallet report with ID: {}", report_id);
            HttpResponse::Ok().json("Successfully deleted wallet report")
        }
        Err(err) => {
            error!(
                "Failed to delete wallet report with ID {}: {:?}",
                report_id, err
            );
            HttpResponse::NotFound().json("Report with supplied id doesn't exist")
        }
    }
}

#[get("/get_wallet_report_metrics/{report_id}")]
async fn get_wallet_report_metrics_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    info!(
        "Received request for wallet report metrics for report ID: {}",
        report_id
    );
    match get_wallet_report_metrics(*report_id) {
        Ok(wallet_metrics) => {
            info!(
                "Successfully retrieved wallet report metrics for report ID: {}",
                report_id
            );
            HttpResponse::Ok().json(wallet_metrics)
        }
        Err(err) => {
            error!(
                "Failed to retrieve wallet report metrics for report ID {}: {:?}",
                report_id, err
            );
            HttpResponse::InternalServerError()
                .json("No wallet report metrics exists for supplied id")
        }
    }
}

#[post("/create_user")]
async fn create_user_endpoint() -> impl Responder {
    info!("Received request to create user");
    match create_user() {
        Ok(api_key) => {
            info!("User created successfully with API key: {}", api_key);
            HttpResponse::Ok().json(api_key)
        }
        Err(err) => {
            error!("Failed to create user: {:?}", err);
            HttpResponse::InternalServerError().json("Unable to create user")
        }
    }
}

#[get("/get_wallet_report_creation_count/{wallet_addr}")]
async fn get_wallet_report_creation_count_endpoint(
    wallet_addr: web::Path<String>,
) -> impl Responder {
    info!(
        "Received wallet report creation count request for wallet address: {}",
        wallet_addr
    );
    match get_wallet_report_count(wallet_addr.clone()) {
        Ok(count) => {
            info!(
                "Successfully retrieved creation count for wallet address: {}. Count: {}",
                wallet_addr, count
            );
            HttpResponse::Ok().json(count)
        }
        Err(err) => {
            error!(
                "Failed to retrieve wallet report creation count for wallet address {}: {:?}",
                wallet_addr, err
            );
            HttpResponse::InternalServerError().json("No wallet report exists for supplied id")
        }
    }
}

#[get("/get_wallet_report_creation_date/{report_id}")]
async fn get_wallet_report_creation_date_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    info!(
        "Received request for wallet report creation date for report ID: {}",
        report_id
    );
    match get_wallet_report_creation_date(*report_id) {
        Ok(creation_date) => {
            info!(
                "Successfully retrieved creation date for report ID: {}",
                report_id
            );
            HttpResponse::Ok().json(creation_date)
        }
        Err(err) => {
            error!(
                "Failed to get creation date for report ID {}: {:?}",
                report_id, err
            );
            HttpResponse::InternalServerError().json("No wallet report exists for supplied id")
        }
    }
}

#[get("/get_wallet_report_case_report/{report_id}")]
async fn get_wallet_report_case_report_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    info!(
        "Received request for wallet report case report for report ID: {}",
        report_id
    );
    match get_wallet_report_case_report(*report_id) {
        Ok(case_report) => {
            info!(
                "Successfully retrieved case report for report ID: {}",
                report_id
            );
            HttpResponse::Ok().json(case_report)
        }
        Err(err) => {
            error!(
                "Failed to get case report for report ID {}: {:?}",
                report_id, err
            );
            HttpResponse::InternalServerError().json("No wallet report exists for supplied id")
        }
    }
}

#[get("/get_wallet_report_score/{report_id}")]
async fn get_wallet_report_score_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    info!(
        "Received request for wallet report score for report ID: {}",
        report_id
    );
    match get_wallet_report_score(*report_id) {
        Ok(score) => {
            info!(
                "Successfully retrieved wallet report score for report ID: {}",
                report_id
            );
            HttpResponse::Ok().json(score)
        }
        Err(err) => {
            error!(
                "Failed to retrieve wallet report score for report ID {}: {:?}",
                report_id, err
            );
            HttpResponse::InternalServerError().json("No wallet report exists for supplied id")
        }
    }
}

#[get("/get_wallet_report_classification/{report_id}")]
async fn get_wallet_report_classification_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    info!(
        "Received request for wallet report classification for report ID: {}",
        report_id
    );
    match get_wallet_report_classification(*report_id) {
        Ok(classification) => {
            info!(
                "Successfully retrieved classification for report ID: {}",
                report_id
            );
            HttpResponse::Ok().json(classification)
        }
        Err(err) => {
            error!(
                "Failed to get wallet report classification for report ID {}: {:?}",
                report_id, err
            );
            HttpResponse::InternalServerError().json("No wallet report exists for supplied id")
        }
    }
}

#[get("/get_wallet_report/{report_id}")]
async fn get_wallet_report_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    info!(
        "Received request for wallet report for report ID: {}",
        report_id
    );
    match get_wallet_report(*report_id) {
        Ok(wallet_report) => {
            info!(
                "Successfully retrieved wallet report for report ID: {}",
                report_id
            );
            HttpResponse::Ok().json(wallet_report)
        }
        Err(err) => {
            error!(
                "Failed to get wallet report for report ID {}: {:?}",
                report_id, err
            );
            HttpResponse::NotFound().json("No wallet report exists for supplied id")
        }
    }
}

#[get("/health")]
async fn health_check() -> impl Responder {
    info!("Health check requested");
    HttpResponse::Ok()
}

#[post("/start_wallet_report/{wallet_addr}")]
async fn start_wallet_report_endpoint(wallet_addr: web::Path<String>) -> impl Responder {
    info!(
        "Received request to start wallet report for wallet address: {}",
        wallet_addr
    );
    let pulsar_client = PulsarClient::new().await;
    let mut pulsar_producer = pulsar_client.create_producer(WALLET_REPUTATION_TOPIC).await;
    let id = Uuid::new_v4();
    let job = WalletReportJob {
        report_id: id,
        wallet_addr: wallet_addr.to_string(),
    };

    match pulsar_producer.enqueue_job(job).await {
        Ok(_) => {
            info!(
                "Successfully enqueued wallet report job with report ID: {}",
                id
            );
            HttpResponse::Ok().json(id)
        }
        Err(err) => {
            error!(
                "Failed to enqueue wallet report job for wallet address {}: {:?}",
                wallet_addr, err
            );
            HttpResponse::InternalServerError().json("Unable to start wallet report")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| {
        App::new()
            .service(health_check)
            .service(get_wallet_report_endpoint)
            .service(start_wallet_report_endpoint)
            .service(get_wallet_report_classification_endpoint)
            .service(get_wallet_report_score_endpoint)
            .service(get_wallet_report_case_report_endpoint)
            .service(get_wallet_report_creation_date_endpoint)
            .service(get_wallet_report_creation_count_endpoint)
            .service(get_wallet_report_metrics_endpoint)
            .service(create_user_endpoint)
            .service(delete_wallet_report_endpoint)
            .service(delete_user_endpoint)
            .service(get_wallet_reports_endpoint)
            .service(get_wallet_reports_by_classification_endpoint)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, App};
    use uuid::Uuid;

    #[actix_web::test]
    async fn test_health_check() {
        // Initialize the app with the health_check endpoint
        let app = test::init_service(App::new().service(health_check)).await;
        // Create a GET request for /health
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        // Expect a 200 OK response
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_wallet_report_creation_date_endpoint() {
        // Initialize the app with the creation date endpoint
        let app =
            test::init_service(App::new().service(get_wallet_report_creation_date_endpoint)).await;
        let uuid = Uuid::new_v4();
        let req = test::TestRequest::get()
            .uri(&format!("/get_wallet_report_creation_date/{}", uuid))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Without proper database setup, this should return an Internal Server Error.
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_get_wallet_report_case_report_endpoint() {
        // Initialize the app with the case report endpoint
        let app =
            test::init_service(App::new().service(get_wallet_report_case_report_endpoint)).await;
        let uuid = Uuid::new_v4();
        let req = test::TestRequest::get()
            .uri(&format!("/get_wallet_report_case_report/{}", uuid))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Expecting fallback error response due to missing dependencies
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_get_wallet_report_score_endpoint() {
        // Initialize the app with the score endpoint
        let app = test::init_service(App::new().service(get_wallet_report_score_endpoint)).await;
        let uuid = Uuid::new_v4();
        let req = test::TestRequest::get()
            .uri(&format!("/get_wallet_report_score/{}", uuid))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Expect error response due to unconfigured database connection
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_get_wallet_report_classification_endpoint() {
        // Initialize the app with the classification endpoint
        let app =
            test::init_service(App::new().service(get_wallet_report_classification_endpoint)).await;
        let uuid = Uuid::new_v4();
        let req = test::TestRequest::get()
            .uri(&format!("/get_wallet_report_classification/{}", uuid))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Expect Internal Server Error response because dependencies are not mocked
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_get_wallet_report_endpoint() {
        // Initialize the app with the wallet report endpoint
        let app = test::init_service(App::new().service(get_wallet_report_endpoint)).await;
        let uuid = Uuid::new_v4();
        let req = test::TestRequest::get()
            .uri(&format!("/get_wallet_report/{}", uuid))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Expect error response because the real database connection is not available
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_start_wallet_report_endpoint() {
        // Initialize the app with the start_wallet_report endpoint
        let app = test::init_service(App::new().service(start_wallet_report_endpoint)).await;
        let fake_wallet_addr = "fake_wallet_address";
        let req = test::TestRequest::post()
            .uri(&format!("/start_wallet_report/{}", fake_wallet_addr))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // As PulsarClient and database dependencies are not configured, we expect an Internal Server Error
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
