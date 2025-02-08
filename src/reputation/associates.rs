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
    pub fn new_from_associates(
        database: &mut Database,
        known_associates: &KnownAssociates,
    ) -> Result<Self> {
        let wallets = database.find_discredited_associates(&known_associates.wallets)?;
        Ok(Self { wallets })
    }
}

pub struct KnownCreditedAssociates {
    pub wallets: Vec<KnownCreditedWallet>,
}

impl KnownCreditedAssociates {
    pub fn new_from_associates(
        database: &mut Database,
        known_associates: &KnownAssociates,
    ) -> Result<Self> {
        let wallets = database.find_credited_associates(&known_associates.wallets)?;
        Ok(Self { wallets })
    }
}

pub struct KnownAssociates {
    pub wallets: Vec<String>,
}

impl KnownAssociates {
    pub fn new(transactions: Vec<EncodedConfirmedTransactionWithStatusMeta>) -> Result<Self> {
        let wallets: Vec<String> = transactions
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

        Ok(Self { wallets })
    }
}
