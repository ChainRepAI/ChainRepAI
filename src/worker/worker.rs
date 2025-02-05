use anyhow::Result;
use futures::TryStreamExt;
use log::{error, info, warn};
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
        info!("Initializing WalletReportWorker...");
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
        info!("Worker started processing jobs.");
        while let Some(msg) = self
            .job_consumer
            .internal_consumer
            .try_next()
            .await
            .expect("Should be able to wait for new message.")
        {
            // Attempt to deserialize the incoming job message.
            let wallet_reputation_job = match msg.deserialize() {
                Ok(data) => data,
                Err(e) => {
                    error!("Couldn't deserialize job, error: {:?}", e);
                    continue;
                }
            };

            // Process the job and acknowledge or negatively acknowledge accordingly.
            match wallet_reputation_job.do_job(self).await {
                Ok(_) => {
                    self.job_consumer.ack(&msg).await;
                    info!("Job processed successfully");
                }
                Err(e) => {
                    self.job_consumer.nack(&msg).await;
                    warn!(
                        "Job processing failed with error: {:?}. Message negatively acknowledged.",
                        e
                    );
                }
            }
        }

        info!("No more messages to process. Exiting worker loop.");
        Ok(())
    }
}
