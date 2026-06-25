use axum::extract::ws::{Message, WebSocket};
use necko3_core::prelude::db::DatabaseExt;
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;
use crate::state::AppState;

pub mod event_loop;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsPaymentDetectedData {
    pub invoice_id: Uuid,
    pub payment_id: Uuid,
    pub tx_hash: String,

    // pub network: String,
    // pub asset: Asset,
    pub amount: String,

    pub from: String,
    pub to: String,

    pub block_number: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WsEvent {
    PaymentDetected(Box<WsPaymentDetectedData>),
    PaymentConfirmed { invoice_id: Uuid, payment_id: Uuid, 
        confirmed_after: u64, },
    PaymentReorged { invoice_id: Uuid, payment_id: Uuid, 
        new_block_number: u64, new_block_hash: String },
    PaymentFailed { invoice_id: Uuid, payment_id: Uuid, },
    PaymentLost { invoice_id: Uuid, payment_id: Uuid, },
    PaymentCancelled { invoice_id: Uuid, payment_id: Uuid, },

    InvoicePaid { invoice_id: Uuid, },
    InvoiceExpired { invoice_id: Uuid, },
    InvoiceCancelled { invoice_id: Uuid, },
}

pub async fn handle_socket<D: DatabaseExt>(mut socket: WebSocket, target_invoice_id: Uuid, state: AppState<D>) {
    let mut rx = state.ws_sender.subscribe();

    while let Ok(event) = rx.recv().await {
        let (inv_id, payload_res) = match &event {
            WsEvent::PaymentDetected(data) => (&data.invoice_id, serde_json::to_string(&event)),
            WsEvent::PaymentConfirmed { invoice_id, .. } => (invoice_id, serde_json::to_string(&event)),
            WsEvent::PaymentReorged { invoice_id, .. } => (invoice_id, serde_json::to_string(&event)),
            WsEvent::PaymentFailed { invoice_id, .. } => (invoice_id, serde_json::to_string(&event)),
            WsEvent::PaymentLost { invoice_id, .. } => (invoice_id, serde_json::to_string(&event)),
            WsEvent::PaymentCancelled { invoice_id, .. } => (invoice_id, serde_json::to_string(&event)),
            WsEvent::InvoicePaid { invoice_id, .. } => (invoice_id, serde_json::to_string(&event)),
            WsEvent::InvoiceExpired { invoice_id, .. } => (invoice_id, serde_json::to_string(&event)),
            WsEvent::InvoiceCancelled { invoice_id, .. } => (invoice_id, serde_json::to_string(&event)),
        };

        if inv_id != &target_invoice_id {
            continue;
        }

        let payload = match payload_res {
            Ok(payload) => payload,
            Err(e) => {
                error!(error = %e, "Failed to serialize WsEvent");
                break;
            }
        };

        if socket.send(Message::Text(payload.into())).await.is_err() {
            break;
        }
    }
}