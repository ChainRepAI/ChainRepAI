use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use uuid::Uuid;
use SolAnalystAI::{
    case_report::case_report::CaseReport, jobs::jobs::WalletReportJob,
    openai_client::openai_client::OpenAIClient, pulsar::pulsar::PulsarClient,
    reputation::reputation::Reputation, solana_client::solana_client::SolanaClient,
    wallet::wallet::Wallet, worker::worker::WALLET_REPUTATION_TOPIC,
};

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/wallet_report/{wallet_addr}")]
async fn wallet_report(wallet_addr: web::Path<String>) -> impl Responder {
    dotenv().ok();
    let solana_client = SolanaClient::new();
    let wallet = Wallet::new(wallet_addr.as_str(), &solana_client).await;
    let reputation = Reputation::new_from_wallet(&wallet);
    let openai_client = OpenAIClient::new();

    match CaseReport::new(&openai_client, reputation, wallet).await {
        Ok(case_report) => HttpResponse::Ok().json(case_report),
        Err(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
    }
}

#[post("/start_wallet_report/{wallet_addr}")]
async fn start_wallet_report(wallet_addr: web::Path<String>) -> impl Responder {
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
            .service(wallet_report)
            .service(start_wallet_report)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
