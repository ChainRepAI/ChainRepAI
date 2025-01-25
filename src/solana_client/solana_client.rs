use solana_client::{
    client_error::ClientError,
    nonblocking::rpc_client::RpcClient,
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

    pub async fn get_transaction_history(
        &self,
        pub_key: &Pubkey,
    ) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>, ClientError> {
        self.client.get_signatures_for_address(pub_key).await
    }

    pub async fn get_token_accounts(
        &self,
        pub_key: &Pubkey,
    ) -> Result<Vec<RpcKeyedAccount>, ClientError> {
        self.client
            .get_token_accounts_by_owner(
                pub_key,
                solana_client::rpc_request::TokenAccountsFilter::ProgramId(*pub_key),
            )
            .await
    }
}
