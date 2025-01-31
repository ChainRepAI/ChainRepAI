use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use SolAnalystAI::{
    case_report::case_report::CaseReport, openai_client::openai_client::OpenAIClient,
    reputation::reputation::Reputation, solana_client::solana_client::SolanaClient,
    wallet::wallet::Wallet,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(health_check).service(wallet_report))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
