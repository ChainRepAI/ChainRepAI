use anyhow::Result;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

use crate::database::{
    models::{KnownCreditedWallet, KnownDiscreditedWallet},
    postgres::Database,
};

pub struct KnownDiscreditedAssociates {
    pub wallets: Vec<KnownDiscreditedWallet>,
}

impl KnownDiscreditedAssociates {
    pub fn new(
        database: &mut Database,
        transactions: Vec<EncodedConfirmedTransactionWithStatusMeta>,
    ) -> Result<Self> {
        let keys: Vec<String> = transactions
            .into_iter()
            .flat_map(|tx| {
                tx.transaction
                    .transaction
                    .decode()
                    .map_or_else(Vec::new, |versioned_tx| {
                        versioned_tx
                            .message
                            .static_account_keys()
                            .iter()
                            .map(|key| key.to_string())
                            .collect()
                    })
            })
            .collect();

        let wallets = database.find_discredited_associates(keys)?;

        Ok(Self { wallets })
    }
}

pub struct KnownCreditedAssociates {
    pub wallets: Vec<KnownCreditedWallet>,
}

impl KnownCreditedAssociates {
    pub fn new(
        database: &mut Database,
        transactions: Vec<EncodedConfirmedTransactionWithStatusMeta>,
    ) -> Result<Self> {
        let keys: Vec<String> = transactions
            .into_iter()
            .flat_map(|tx| {
                tx.transaction
                    .transaction
                    .decode()
                    .map_or_else(Vec::new, |versioned_tx| {
                        versioned_tx
                            .message
                            .static_account_keys()
                            .iter()
                            .map(|key| key.to_string())
                            .collect()
                    })
            })
            .collect();

        let wallets = database.find_credited_associates(keys)?;

        Ok(Self { wallets })
    }
}

pub struct KnownAssociates {
    wallets: Vec<String>,
}
