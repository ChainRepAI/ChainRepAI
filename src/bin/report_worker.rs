use dotenv::dotenv;
use SolAnalystAI::worker::worker::WalletReportWorker;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mut worker = WalletReportWorker::new().await;
    worker.do_work().await.unwrap();
}
