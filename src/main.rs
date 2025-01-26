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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(health_check))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
