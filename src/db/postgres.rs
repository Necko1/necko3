use crate::config::{ChainConfig, TokenConfig};
use crate::db::DatabaseAdapter;
use crate::model::{Invoice, InvoiceStatus};
use alloy::primitives::U256;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sqlx::PgPool;

pub struct Postgres {
    pool: PgPool,
    chains: RwLock<HashMap<String, Arc<ChainConfig>>>,
}

impl Postgres {
    pub async fn init(pool: PgPool) -> anyhow::Result<Self> {
        let db = Self { pool, chains: RwLock::new(HashMap::new()) };

        // todo...

        Ok(db)
    }
}

impl DatabaseAdapter for Postgres {
    async fn get_chains(&self) -> anyhow::Result<HashMap<String, ChainConfig>> {
        todo!()
    }

    async fn get_chain(&self, chain_name: &str) -> anyhow::Result<Option<ChainConfig>> {
        todo!()
    }

    async fn get_chain_by_id(&self, id: u32) -> anyhow::Result<Option<ChainConfig>> {
        todo!()
    }

    async fn add_chain(&self, chain_config: &ChainConfig) -> anyhow::Result<()> {
        todo!()
    }

    async fn update_chain_block(&self, chain_name: &str, block_num: u64) -> anyhow::Result<()> {
        todo!()
    }

    async fn get_latest_block(&self, chain_name: &str) -> anyhow::Result<u64> {
        todo!()
    }

    async fn get_chains_with_token(&self, token_symbol: &str) -> anyhow::Result<Vec<ChainConfig>> {
        todo!()
    }

    async fn remove_chain(&self, chain_name: &str) -> anyhow::Result<()> {
        todo!()
    }

    async fn remove_chain_by_id(&self, id: u32) -> anyhow::Result<()> {
        todo!()
    }

    async fn get_watch_addresses(&self, chain_name: &str) -> anyhow::Result<Vec<String>> {
        todo!()
    }

    async fn remove_watch_address(&self, chain_name: &str, address: &str) -> anyhow::Result<()> {
        todo!()
    }

    async fn remove_watch_addresses_bulk<S: AsRef<str>>(&self, chain_name: &str, addresses: &[S]) -> anyhow::Result<()> {
        todo!()
    }

    async fn add_watch_address(&self, chain_name: &str, address: &str) -> anyhow::Result<()> {
        todo!()
    }

    async fn get_xpub(&self, chain_name: &str) -> anyhow::Result<String> {
        todo!()
    }

    async fn get_rpc_url(&self, chain_name: &str) -> anyhow::Result<String> {
        todo!()
    }

    async fn get_tokens(&self, chain_name: &str) -> anyhow::Result<Vec<TokenConfig>> {
        todo!()
    }

    async fn get_token_contracts(&self, chain_name: &str) -> anyhow::Result<Vec<String>> {
        todo!()
    }

    async fn get_token(&self, chain_name: &str, token_symbol: &str) -> anyhow::Result<Option<TokenConfig>> {
        todo!()
    }

    async fn get_token_by_id(&self, chain_name: &str, id: u32) -> anyhow::Result<Option<TokenConfig>> {
        todo!()
    }

    async fn get_token_by_contract(&self, chain_name: &str, contract_address: &str) -> anyhow::Result<Option<TokenConfig>> {
        todo!()
    }

    async fn remove_token(&self, chain_name: &str, token_symbol: &str) -> anyhow::Result<()> {
        todo!()
    }

    async fn remove_token_by_id(&self, chain_name: &str, id: u32) -> anyhow::Result<()> {
        todo!()
    }

    async fn add_token(&self, chain_name: &str, token_config: &TokenConfig) -> anyhow::Result<()> {
        todo!()
    }

    async fn get_invoices(&self) -> anyhow::Result<Vec<Invoice>> {
        todo!()
    }

    async fn get_invoices_by_chain(&self, chain_name: &str) -> anyhow::Result<Vec<Invoice>> {
        todo!()
    }

    async fn get_invoices_by_token(&self, token_symbol: &str) -> anyhow::Result<Vec<Invoice>> {
        todo!()
    }

    async fn get_invoices_by_address(&self, address: &str) -> anyhow::Result<Vec<Invoice>> {
        todo!()
    }

    async fn get_invoice(&self, uuid: &str) -> anyhow::Result<Option<Invoice>> {
        todo!()
    }

    async fn get_invoices_by_status(&self, status: InvoiceStatus) -> anyhow::Result<Vec<Invoice>> {
        todo!()
    }

    async fn get_invoices_by_chain_and_status(&self, chain_name: &str, status: InvoiceStatus) -> anyhow::Result<Vec<Invoice>> {
        todo!()
    }

    async fn get_invoices_by_address_and_status(&self, address: &str, status: InvoiceStatus) -> anyhow::Result<Vec<Invoice>> {
        todo!()
    }

    async fn get_busy_indexes(&self, chain_name: &str) -> anyhow::Result<Vec<u32>> {
        todo!()
    }

    async fn add_invoice(&self, invoice: &Invoice) -> anyhow::Result<()> {
        todo!()
    }

    async fn set_invoice_status(&self, uuid: &str, status: InvoiceStatus) -> anyhow::Result<()> {
        todo!()
    }

    async fn add_payment(&self, uuid: &str, amount_raw: U256) -> anyhow::Result<(U256, String)> {
        todo!()
    }

    async fn get_pending_invoice_by_address(&self, chain_name: &str, address: &str) -> anyhow::Result<Option<Invoice>> {
        todo!()
    }

    async fn expire_old_invoices(&self) -> anyhow::Result<Vec<(String, String, String)>> {
        todo!()
    }

    async fn is_invoice_expired(&self, uuid: &str) -> anyhow::Result<bool> {
        todo!()
    }

    async fn is_invoice_paid(&self, uuid: &str) -> anyhow::Result<bool> {
        todo!()
    }

    async fn is_invoice_pending(&self, uuid: &str) -> anyhow::Result<bool> {
        todo!()
    }

    async fn remove_invoice(&self, uuid: &str) -> anyhow::Result<()> {
        todo!()
    }

    async fn get_token_decimals(&self, chain_name: &str, token_symbol: &str) -> anyhow::Result<u8> {
        todo!()
    }
}