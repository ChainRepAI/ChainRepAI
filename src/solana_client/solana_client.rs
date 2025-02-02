use futures::future::join_all;
use solana_client::{
    client_error::ClientError,
    nonblocking::rpc_client::RpcClient,
    rpc_response::{
        RpcConfirmedTransactionStatusWithSignature, RpcKeyedAccount, RpcPrioritizationFee,
    },
};
use solana_sdk::{account::Account, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{env, str::FromStr};

const CHUNK_SIZE: usize = 100;

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

    pub async fn get_transaction(
        &self,
        signature: String,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta, ClientError> {
        let signature =
            Signature::from_str(&signature).expect("Should be able to create signature");
        self.client
            .get_transaction(&signature, UiTransactionEncoding::Json)
            .await
    }

    pub async fn get_recent_prioritization_fees(
        &self,
        pub_key: &Pubkey,
    ) -> Result<Vec<RpcPrioritizationFee>, ClientError> {
        self.client
            .get_recent_prioritization_fees(&[*pub_key])
            .await
    }

    pub async fn batch_process_transactions(
        &self,
        signatures: Vec<RpcConfirmedTransactionStatusWithSignature>,
    ) -> Vec<EncodedConfirmedTransactionWithStatusMeta> {
        let mut confirmed_transactions = Vec::new();

        // Process the signatures in defined chunk sizes.
        for chunk in signatures.chunks(CHUNK_SIZE) {
            // Create a batch of asynchronous tasks for parallel processing.
            let futures = chunk.iter().map(|sig_info| {
                // Clone the signature string to move it into the async block.
                let sig_clone = sig_info.signature.clone();
                // An async move block to ensure the signature's lifetime is properly contained.
                async move {
                    let signature = Signature::from_str(&sig_clone)
                        .expect("Invalid signature format");
                    self.client
                        .get_transaction(&signature, UiTransactionEncoding::Json)
                        .await
                }
            });

            // Await all tasks concurrently and filter out failures.
            let results: Vec<_> = join_all(futures)
                .await
                .into_iter()
                .filter_map(|tx_result| tx_result.ok())
                .collect();

            confirmed_transactions.extend(results);
        }

        confirmed_transactions
    }
}
}
