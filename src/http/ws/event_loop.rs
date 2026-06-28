use necko3_core::prelude::db::traits::DatabaseExt;
use necko3_core::types::{CoreEvent, ExternalEvent, NeckoEvent};
use necko3_core::types::core::{Invoice, Payment};
use tracing::{debug, error, info};
use uuid::Uuid;
use crate::http::ws::{WsEvent, WsPaymentDetectedData};
use crate::state::AppState;

pub struct EventForwarder<D> {
    state: AppState<D>
}

impl<D: DatabaseExt> EventForwarder<D> {
    pub fn new(state: AppState<D>) -> Self {
        Self { state }
    }

    pub async fn run(self) {
        let (_id, mut rx) = self.state.core.subscribe(5000);

        info!("Event forwarder started");

        while let Some(necko_event) = rx.recv().await {
            let ws_event_opt = match necko_event {
                NeckoEvent::Core(event) => self.match_core_event(event).await,
                NeckoEvent::Ext(event) => self.match_ext_event(event).await,
            };

            let Some(ws_event) = ws_event_opt else { continue };

            let _ = self.state.ws_sender.send(ws_event);
        }
    }

    async fn match_core_event(&self, core_event: CoreEvent) -> Option<WsEvent> {
        let event = match core_event {
            CoreEvent::TransactionDetected(data) => {
                let invoice = self.get_invoice_by_address(&data.to).await?;

                WsEvent::PaymentDetected(Box::new(WsPaymentDetectedData {
                    invoice_id: invoice.id,
                    payment_id: data.db_transaction_id,
                    tx_hash: data.tx_hash,
                    amount: data.amount_human,
                    from: data.from,
                    to: data.to,
                    block_number: data.block_number,
                }))
            }
            CoreEvent::TransactionConfirmed { db_transaction_id, confirmed_after, .. } => {
                let payment = self.get_payment_by_id(db_transaction_id).await?;
                let invoice = self.get_invoice_by_address(&payment.to).await?;

                WsEvent::PaymentConfirmed {
                    invoice_id: invoice.id,
                    payment_id: payment.id,
                    confirmed_after,
                }
            }
            CoreEvent::TransactionReorged { db_transaction_id, new_block_number,
                new_block_hash, .. } => {
                let payment = self.get_payment_by_id(db_transaction_id).await?;
                let invoice = self.get_invoice_by_address(&payment.to).await?;

                WsEvent::PaymentReorged {
                    invoice_id: invoice.id,
                    payment_id: payment.id,
                    new_block_number,
                    new_block_hash,
                }
            }
            CoreEvent::TransactionFailed { db_transaction_id, .. } => {
                let payment = self.get_payment_by_id(db_transaction_id).await?;
                let invoice = self.get_invoice_by_address(&payment.to).await?;

                WsEvent::PaymentFailed {
                    invoice_id: invoice.id,
                    payment_id: payment.id,
                }
            }
            CoreEvent::TransactionLost { tx_hash } => {
                let payment = self.get_payment_by_tx_hash(tx_hash).await?;
                let invoice = self.get_invoice_by_address(&payment.to).await?;

                WsEvent::PaymentLost {
                    invoice_id: invoice.id,
                    payment_id: payment.id,
                }
            }
        };

        Some(event)
    }

    async fn match_ext_event(&self, ext_event: ExternalEvent) -> Option<WsEvent> {
        let event = match ext_event {
            ExternalEvent::InvoicePaid { invoice_id } => WsEvent::InvoicePaid { invoice_id },
            ExternalEvent::InvoiceExpired { invoice_id } => WsEvent::InvoiceExpired { invoice_id },
            ExternalEvent::InvoiceCancelled { invoice_id } => WsEvent::InvoiceCancelled { invoice_id },

            _ => return None,
        };
        
        Some(event)
    }
}

impl<D: DatabaseExt> EventForwarder<D> {
    async fn get_invoice_by_address(&self, address: &str) -> Option<Invoice> {
        let invoice_opt = self.state.core.db().get_invoice_by_address(&address).await
            .unwrap_or_else(|e| {
                error!("DB error while getting invoice for address {}: {}", address, e);
                None
            });

        let Some(invoice) = invoice_opt else {
            debug!("No matching invoice for address {} found", address);
            return None
        };

        Some(invoice)
    }

    async fn get_payment_by_id(&self, payment_id: Uuid) -> Option<Payment> {
        let payment_opt = self.state.core.db().get_payment(payment_id).await
            .unwrap_or_else(|e| {
                error!("DB error while getting payment for id {}: {}", payment_id, e);
                None
            });

        let Some(payment) = payment_opt else {
            debug!("No matching payment for id {} found", payment_id);
            return None
        };

        Some(payment)
    }

    async fn get_payment_by_tx_hash(&self, tx_hash: String) -> Option<Payment> {
        let payment_opt = self.state.core.db()
            .get_payment_by_tx_hash(&tx_hash).await
            .unwrap_or_else(|e| {
                error!("DB error while getting payment for tx_hash {}: {}", tx_hash, e);
                None
            });

        let Some(payment) = payment_opt else {
            debug!("No matching payment for tx_hash {} found", tx_hash);
            return None
        };

        Some(payment)
    }
}