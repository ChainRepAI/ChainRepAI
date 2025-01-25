use solana_client::{
    client_error::ClientError,
    nonblocking::rpc_client::RpcClient,
    rpc_request::TokenAccountsFilter,
    rpc_response::{RpcConfirmedTransactionStatusWithSignature, RpcKeyedAccount},
};
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::env;

pub struct SolanaClient {
    client: RpcClient,
}
impl SolanaClient {
    pub fn new() -> Self {
        Self {
            client: RpcClient::new(env::var("RPC_URL").expect("rpc url should be set")),
        }
    }
    pub async fn get_account_balance(&self, pub_key: &Pubkey) -> Result<u64, ClientError> {
        self.client.get_balance(pub_key).await
    }
    pub async fn get_account_info(&self, pub_key: &Pubkey) -> Result<Account, ClientError> {
        self.client.get_account(pub_key).await
    }
}
