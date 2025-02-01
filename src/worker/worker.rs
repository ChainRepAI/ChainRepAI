use crate::{database::postgres::Database, openai_client::openai_client::OpenAIClient, solana_client::solana_client::SolanaClient};

pub struct WalletReportWorker {
    database: Database,
    solana_client: SolanaClient,
    openai_client: OpenAIClient,
}

