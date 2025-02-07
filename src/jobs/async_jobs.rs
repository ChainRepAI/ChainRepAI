use anyhow::Result;
use log::info;
use pulsar::{producer, DeserializeMessage, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    case_report::case_report::CaseReport, database::models::{KnownDiscreditedWallet, WalletReport},
    reputation::reputation::Reputation, wallet::wallet::Wallet, worker::worker::WalletReportWorker,
};

const DISCREDITED_SCORE_RATING_BOUNDARY: i32 = 400;

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletReportJob {
    pub report_id: Uuid,
    pub wallet_addr: String,
}

impl SerializeMessage for WalletReportJob {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for WalletReportJob {
    type Output = Result<WalletReportJob, serde_json::Error>;

    fn deserialize_message(payload: &pulsar::Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

impl WalletReportJob {
    pub async fn do_job(&self, worker: &mut WalletReportWorker) -> Result<()> {
        info!(
            "Starting WalletReportJob for wallet address: {} and report_id: {}",
            self.wallet_addr, self.report_id
        );

        let wallet = Wallet::new(self.wallet_addr.as_str(), &worker.solana_client).await;
        info!("Wallet retrieved: {:?}", wallet);

        let reputation = Reputation::new_from_wallet(&wallet, self.report_id.clone());
        info!(
            "Computed reputation for report_id {}: rating_classification = {:?}, rating_score = {}",
            self.report_id, reputation.rating_classification, reputation.rating_score
        );

        // Generate case report
        let case_report = CaseReport::new(&worker.openai_client, &reputation, wallet).await?;
        info!("Generated case report for wallet: {}", self.wallet_addr);

        // Create wallet report
        let wallet_report = WalletReport::new(
            self.report_id,
            reputation.rating_classification,
            reputation.rating_score,
            case_report,
            self.wallet_addr.clone(),
        )?;
        info!("Wallet report created, proceeding to database insertion");

        // Insert wallet report into database
        worker.database.insert_wallet_report(wallet_report)?;
        info!(
            "Wallet report inserted successfully for report_id {}",
            self.report_id
        );

        // Insert wallet metrics into database
        worker
            .database
            .insert_wallet_metrics(reputation.wallet_metrics)?;
        info!(
            "Wallet metrics inserted successfully for wallet: {}",
            self.wallet_addr
        );

        if reputation.rating_score < DISCREDITED_SCORE_RATING_BOUNDARY {
            worker.database.insert_discredited_wallet(KnownDiscreditedWallet::new(self.wallet_addr.clone()))?
        }

        info!(
            "Finished WalletReportJob for wallet address: {}",
            self.wallet_addr
        );
        Ok(())
    }
}
