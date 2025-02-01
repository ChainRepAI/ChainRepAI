use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use dotenv::dotenv;
use uuid::Uuid;
use SolAnalystAI::{
    case_report::case_report::CaseReport,
    database::{
        models::{RatingClassification, WalletReport},
        postgres::Database,
    },
    jobs::jobs::WalletReportJob,
    pulsar::pulsar::PulsarClient,
    worker::worker::WALLET_REPUTATION_TOPIC,
};

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
