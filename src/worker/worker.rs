use pulsar::SubType;

use crate::{
    database::postgres::Database,
    openai_client::openai_client::OpenAIClient,
    pulsar::pulsar::{PulsarClient, PulsarConsumer},
    solana_client::solana_client::SolanaClient,
};

pub const WALLET_REPUTATION_TOPIC: &str = "non-persistent://public/default/wallet-reputation";
const SUB: &str = "wallet-reputation-sub";
const SUB_TYPE: SubType = SubType::Shared;

pub struct WalletReportWorker {
    database: Database,
    solana_client: SolanaClient,
    openai_client: OpenAIClient,
    job_consumer: PulsarConsumer,
}

impl WalletReportWorker {
    pub async fn new() -> Self {
        let pulsar_client = PulsarClient::new().await;
        Self {
            database: Database::connect().expect("Should be able to connect to db"),
            solana_client: SolanaClient::new(),
            openai_client: OpenAIClient::new(),
            job_consumer: pulsar_client
                .create_consumer(vec![WALLET_REPUTATION_TOPIC], SUB_TYPE, SUB)
                .await,
        }
    }
}
