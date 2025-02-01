use anyhow::Result;
use pulsar::{producer, DeserializeMessage, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    case_report::case_report::CaseReport, database::models::WalletReport,
    reputation::reputation::Reputation, wallet::wallet::Wallet, worker::worker::WalletReportWorker,
};

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
        let wallet = Wallet::new(self.wallet_addr.as_str(), &worker.solana_client).await;
        let reputation = Reputation::new_from_wallet(&wallet);
        let case_report = CaseReport::new(&worker.openai_client, &reputation, wallet).await?;
        let wallet_report = WalletReport::new(
            self.report_id,
            reputation.rating_classification,
            reputation.rating_score,
            case_report,
        )?;
        worker.database.insert_wallet_report(wallet_report)?;
        Ok(())
    }
}
