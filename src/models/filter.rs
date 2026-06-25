use necko3_core::types::core::{InvoiceStatus, PaymentStatus, WebhookStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct QueryInvoiceFilter {
    pub status: Option<InvoiceStatus>,
    pub address: Option<String>,
    pub network: Option<String>,
    pub token: Option<String>,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct QueryPaymentFilter {
    pub from: Option<String>,
    pub to: Option<String>,
    pub network: Option<String>,
    pub token: Option<String>,
    pub block_number: Option<u64>,
    pub block_hash: Option<String>,
    pub status: Option<PaymentStatus>,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct QueryWebhookFilter {
    pub invoice_id: Option<Uuid>,
    /// WebhookEvent::to_string()
    pub event_type: Option<String>,
    pub url: Option<String>,
    pub status: Option<WebhookStatus>,
}