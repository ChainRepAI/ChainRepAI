use solana_client::rpc_response::{RpcConfirmedTransactionStatusWithSignature, RpcKeyedAccount};
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::solana_client::solana_client::SolanaClient;

pub struct Wallet {
    pub_key: Pubkey,
    account_balance: u64,
    account_info: Account,
    transaction_history: Vec<RpcConfirmedTransactionStatusWithSignature>,
    token_accounts: Vec<RpcKeyedAccount>,
}

impl Wallet {
    pub async fn new(wallet_addr: &str, solana_client: &SolanaClient) -> Self {
        let pub_key = Pubkey::from_str_const(wallet_addr);

        let (account_balance, account_info, transaction_history, token_accounts) = tokio::join!(
            solana_client.get_account_balance(&pub_key),
            solana_client.get_account_info(&pub_key),
            solana_client.get_transaction_history(&pub_key),
            solana_client.get_token_accounts(&pub_key),
        );

        Self {
            pub_key,
            account_balance: account_balance.unwrap_or_default(),
            account_info: account_info.unwrap_or_default(),
            transaction_history: transaction_history.unwrap_or_default(),
            token_accounts: token_accounts.unwrap_or_default(),
        }
    }
}
