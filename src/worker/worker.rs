use anyhow::Result;
use futures::TryStreamExt;
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
    pub database: Database,
    pub solana_client: SolanaClient,
    pub openai_client: OpenAIClient,
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

    pub async fn do_work(&mut self) -> Result<()> {
        while let Some(msg) = self
            .job_consumer
            .internal_consumer
            .try_next()
            .await
            .expect("Should be able to wait for new message.")
        {
            let wallet_reputation_job = match msg.deserialize() {
                Ok(data) => data,
                Err(e) => {
                    println!("Couldn't deseralize job, error: {:?}.", e);
                    continue;
                }
            };

            match wallet_reputation_job.do_job(self).await {
                Ok(_) => {
                    self.job_consumer.ack(&msg).await;
                    println!("Job processed successfully");
                }
                Err(_) => {
                    self.job_consumer.nack(&msg).await;
                    println!("Couldn't process job successfully");
                }
            }
        }

        Ok(())
    }
}
