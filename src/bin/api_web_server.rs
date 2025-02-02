use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use chrono::NaiveDateTime;
use dotenv::dotenv;
use uuid::Uuid;
use SolAnalystAI::{
    case_report::case_report::CaseReport,
    database::{
        models::{RatingClassification, WalletMetrics, WalletReport},
        postgres::Database,
    },
    jobs::jobs::WalletReportJob,
    pulsar::pulsar::PulsarClient,
    worker::worker::WALLET_REPUTATION_TOPIC,
};

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
    Ok(database.get_wallet_report(report_id)?)
}

#[get("/get_wallet_report_metrics/{report_id}")]
async fn get_wallet_report_metrics_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    match get_wallet_report_metrics(*report_id) {
        Ok(wallet_metrics) => HttpResponse::Ok().json(wallet_metrics),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
    }
}

#[get("/get_wallet_report_creation_count/{wallet_addr}")]
async fn get_wallet_report_creation_count_endpoint(
    wallet_addr: web::Path<String>,
) -> impl Responder {
    match get_wallet_report_count(wallet_addr.clone()) {
        Ok(count) => HttpResponse::Ok().json(count),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
    }
}

#[get("/get_wallet_report_creation_date/{report_id}")]
async fn get_wallet_report_creation_date_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    match get_wallet_report_creation_date(*report_id) {
        Ok(creation_date) => HttpResponse::Ok().json(creation_date),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
    }
}

#[get("/get_wallet_report_case_report/{report_id}")]
async fn get_wallet_report_case_report_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    match get_wallet_report_case_report(*report_id) {
        Ok(case_report) => HttpResponse::Ok().json(case_report),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
    }
}

#[get("/get_wallet_report_score/{report_id}")]
async fn get_wallet_report_score_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    match get_wallet_report_score(*report_id) {
        Ok(score) => HttpResponse::Ok().json(score),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
    }
}

#[get("/get_wallet_report_classification/{report_id}")]
async fn get_wallet_report_classification_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    match get_wallet_report_classification(*report_id) {
        Ok(classification) => HttpResponse::Ok().json(classification),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
    }
}

#[get("/get_wallet_report/{report_id}")]
async fn get_wallet_report_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    match get_wallet_report(*report_id) {
        Ok(wallet_report) => HttpResponse::Ok().json(wallet_report),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
    }
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/start_wallet_report/{wallet_addr}")]
async fn start_wallet_report_endpoint(wallet_addr: web::Path<String>) -> impl Responder {
    let pulsar_client = PulsarClient::new().await;
    let mut pulsar_producer = pulsar_client.create_producer(WALLET_REPUTATION_TOPIC).await;
    let id = Uuid::new_v4();
    let job = WalletReportJob {
        report_id: id,
        wallet_addr: wallet_addr.to_string(),
    };

    match pulsar_producer.enqueue_job(job).await {
        Ok(_) => HttpResponse::Ok().json(id),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
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
