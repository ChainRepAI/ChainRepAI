use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::Serialize;
use ChainRepAI::{reputation::reputation::{Reputation, ReputationItem, ReputationLevel}, solana_client::solana_client::SolanaClient, wallet::wallet::Wallet};

#[derive(Serialize)]
struct ReputationResponse {
    wallet_addr: String,
    reputation_items: Vec<ReputationItem>,
    average_reputation: ReputationLevel,
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/wallet_reputation/{wallet_addr}")]
async fn wallet_reputation(wallet_addr: web::Path<String>) -> impl Responder {
    dotenv().ok();
    let solana_client = SolanaClient::new();
    let wallet = Wallet::new(wallet_addr.as_str(), &solana_client).await;
    let mut reputation = Reputation::new();
    reputation.calc_reputation(&wallet);
    let reputation_response = ReputationResponse {
        wallet_addr: wallet_addr.to_string(),
        reputation_items: reputation.items,
        average_reputation: ReputationLevel::Medium,
    };
    HttpResponse::Ok().json(reputation_response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(health_check).service(wallet_reputation))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
