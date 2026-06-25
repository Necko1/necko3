use chrono::{DateTime, Utc};
use necko3_core::types::core::{Invoice, InvoiceStatus, Payment, PaymentStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicInvoiceModel {
    pub id: Uuid,
    pub address: String,
    pub amount: String,
    pub paid: String,
    pub token: String,
    pub network: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: InvoiceStatus,
}

impl From<Invoice> for PublicInvoiceModel {
    fn from(value: Invoice) -> Self {
        Self {
            id: value.id,
            address: value.address,
            amount: value.amount,
            paid: value.paid,
            token: value.token,
            network: value.network,
            created_at: value.created_at,
            expires_at: value.expires_at,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicPaymentModel {
    pub id: Uuid,
    pub from: String,
    pub to: String,
    pub network: String,
    pub token: String,
    pub tx_hash: String,
    pub amount: String,
    pub block_number: u64,
    pub block_hash: String,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
}

impl PublicPaymentModel {
    pub fn from(payment: Payment, amount_human: String) -> Self {
        Self {
            id: payment.id,
            from: payment.from,
            to: payment.to,
            network: payment.network,
            token: payment.token,
            tx_hash: payment.tx_hash,
            amount: amount_human,
            block_number: payment.block_number,
            block_hash: payment.block_hash,
            status: payment.status,
            created_at: payment.created_at,
        }
    } 
}