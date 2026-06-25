use necko3_core::builder::invoice_config::WebhookConfig;
use necko3_core::types::core::db::Pagination;
use serde_aux::prelude::deserialize_number_from_string;
use serde::{Deserialize, Serialize};
use crate::auth::Permission;
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
    900
}