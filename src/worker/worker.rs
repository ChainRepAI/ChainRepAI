use crate::{database::postgres::Database, openai_client::openai_client::OpenAIClient, solana_client::solana_client::SolanaClient};

pub struct WalletReportWorker {
    database: Database,
    solana_client: SolanaClient,
    openai_client: OpenAIClient,
}

impl WalletReportWorker {
    pub fn new() -> Self {
        Self {
            database: Database::connect().expect("Should be able to connect to db"),
            solana_client: SolanaClient::new(),
            openai_client: OpenAIClient::new(),
        }
    }
}