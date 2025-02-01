use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use dotenv::dotenv;
use uuid::Uuid;
use SolAnalystAI::{
    database::{models::WalletReport, postgres::Database},
    jobs::jobs::WalletReportJob,
    pulsar::pulsar::PulsarClient,
    worker::worker::WALLET_REPUTATION_TOPIC,
};

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

fn get_wallet_report(report_id: Uuid) -> Result<WalletReport> {
    let mut database = Database::connect()?;
    Ok(database.get_wallet_report(report_id)?)
}

#[get("/get_wallet_report/{report_id}")]
async fn get_wallet_report_endpoint(report_id: web::Path<Uuid>) -> impl Responder {
    dotenv().ok();
    match get_wallet_report(*report_id) {
        Ok(wallet_report) => HttpResponse::Ok().json(wallet_report),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
    }
}

#[post("/start_wallet_report/{wallet_addr}")]
async fn start_wallet_report_endpoint(wallet_addr: web::Path<String>) -> impl Responder {
    dotenv().ok();

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
    HttpServer::new(|| {
        App::new()
            .service(health_check)
            .service(get_wallet_report_endpoint)
            .service(start_wallet_report_endpoint)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
