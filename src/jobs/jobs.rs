use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct WalletReportJob {
    wallet_addr: String,
}
