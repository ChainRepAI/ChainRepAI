use solana_client::rpc_response::{RpcConfirmedTransactionStatusWithSignature, RpcKeyedAccount};
use solana_sdk::{account::Account, pubkey::Pubkey};

pub struct Wallet {
    pub_key: Pubkey,
    account_balance: u64,
    account_info: Account,
    transaction_history: Vec<RpcConfirmedTransactionStatusWithSignature>,
    token_accounts: Vec<RpcKeyedAccount>,
}
