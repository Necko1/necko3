use axum::extract::ws::{Message, WebSocket};
use necko3_core::prelude::db::traits::DatabaseExt;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::RecvError;
use tracing::{error, warn};
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

    InvoicePaid { invoice_id: Uuid, },
    InvoiceExpired { invoice_id: Uuid, },
    InvoiceCancelled { invoice_id: Uuid, },
}

pub async fn handle_socket<D: DatabaseExt>(mut socket: WebSocket, target_invoice_id: Uuid, state: AppState<D>) {
    let mut rx = state.ws_sender.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => match msg {
                Ok(event) => {
                    let inv_id = match &event {
                        WsEvent::PaymentDetected(data) => &data.invoice_id,
                        WsEvent::PaymentConfirmed { invoice_id, .. } => invoice_id,
                        WsEvent::PaymentReorged { invoice_id, .. } => invoice_id,
                        WsEvent::PaymentFailed { invoice_id, .. } => invoice_id,
                        WsEvent::PaymentLost { invoice_id, .. } => invoice_id,
                        WsEvent::InvoicePaid { invoice_id, .. } => invoice_id,
                        WsEvent::InvoiceExpired { invoice_id, .. } => invoice_id,
                        WsEvent::InvoiceCancelled { invoice_id, .. } => invoice_id,
                    };

                    if inv_id != &target_invoice_id {
                        continue;
                    }

                    let payload = match serde_json::to_string(&event) {
                        Ok(p) => p,
                        Err(e) => {
                            error!(error = %e, "Failed to serialize WsEvent");
                            continue;
                        }
                    };

                    if socket.send(Message::Text(payload.into())).await.is_err() {
                        break;
                    }
                }
                Err(RecvError::Lagged(skipped)) => {
                    warn!("Client skipped {} messages", skipped);
                    continue;
                }
                Err(RecvError::Closed) => {
                    break;
                }
            },

            client_msg = socket.recv() => {
                match client_msg {
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}