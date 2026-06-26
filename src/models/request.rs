use std::collections::HashSet;
use necko3_core::builder::chain_config::ChainConfig;
use necko3_core::builder::invoice_config::WebhookConfig;
use necko3_core::builder::token_config::TokenConfig;
use necko3_core::types::core::ChainType;
use necko3_core::types::core::db::Pagination;
use serde_aux::prelude::deserialize_number_from_string;
use serde::{Deserialize, Serialize};
use crate::models::Permission;
use crate::models::ApiKeyPrefix;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct QueryPagination {
    #[serde(default = "default_page_size")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub page_size: u32,

    #[serde(default = "default_page")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub page: u64,
}

impl Default for QueryPagination {
    fn default() -> Self {
        Self {
            page_size: default_page_size(),
            page: default_page(),
        }
    }
}

fn default_page_size() -> u32 { 20 }
fn default_page() -> u64 { 1 }

impl From<QueryPagination> for Pagination {
    fn from(value: QueryPagination) -> Self {
        let page = if value.page == 0 { 1 } else { value.page };

        let limit = value.page_size.min(100);

        Pagination {
            limit,
            offset: (page - 1) * limit as u64,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateKeyReq {
    pub name: String,
    pub prefix: ApiKeyPrefix,
    pub permissions: Vec<Permission>,
}

#[derive(Deserialize)]
pub struct CreateInvoiceReq {
    pub network: String,
    pub asset: String, // native/token
    pub amount: f64,
    #[serde(default = "default_duration")]
    pub duration: u64, // seconds
    pub webhook_config: Option<WebhookConfig>,
}

fn default_duration() -> u64 {
    3600
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChainReq {
    pub name: String,
    pub active: bool,
    pub rpc_urls: Vec<String>,
    pub chain_type: ChainType,
    pub xpub: String,
    pub native_symbol: String,
    pub decimals: u8,
    pub last_processed_block: u64,
    pub block_lag: u8,
    pub safe_lag: u8,
    pub required_confirmations: u64,
    pub logo_url: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<TokenConfig>,
}

impl From<CreateChainReq> for ChainConfig {
    fn from(value: CreateChainReq) -> Self {
        Self {
            name: value.name,
            active: value.active,
            rpc_urls: value.rpc_urls,
            chain_type: value.chain_type,
            xpub: value.xpub,
            native_symbol: value.native_symbol,
            decimals: value.decimals,
            last_processed_block: value.last_processed_block,
            block_lag: value.block_lag,
            safe_lag: value.safe_lag,
            required_confirmations: value.required_confirmations,
            logo_url: value.logo_url,
            watch_addresses: HashSet::new(),
            tokens: value.tokens,
        }
    }
}

#[derive(Deserialize)]
pub struct ImageProxyParams {
    pub url: String,
}