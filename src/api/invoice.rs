use crate::chain::{Blockchain, BlockchainAdapter};
use crate::db::DatabaseAdapter;
use crate::model::{CreateInvoiceReq, Invoice, InvoiceStatus};
use crate::state::AppState;
use alloy::primitives::utils::parse_units;
use alloy::primitives::U256;
use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;

pub async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateInvoiceReq>,
) -> String {
    let chain_config = match state.db.get_chain(&payload.network).await {
        Ok(occ) => match occ { 
            Some(cc) => cc,
            None => return format!("Error: network '{}' is not currently supported", payload.network)
        },
        Err(e) => return e.to_string()
    };

    let token_decimals = match state.db.get_token_decimals(&payload.network, &payload.token).await { 
        Ok(dec) => dec,
        Err(e) => return e.to_string()
    };

    let amount_raw = match parse_units(&payload.amount, token_decimals) {
        Ok(a) => a,
        Err(e) => {
            return format!("Error while trying to parse units: {}", e)
        }
    };

    let Some(index) = state.get_free_slot(&payload.network).await else {
        return "Error: no free slots available".into();
    };


    let blockchain = Blockchain::new(state.clone(), chain_config.chain_type,
                                     &payload.network, None);
    let address = match blockchain.derive_address(index).await {
        Ok(a) => a,
        Err(e) => {
            return format!("Error: failed to get address (index {}) for {} chain: {}",
                           index, chain_config.chain_type, e);
        }
    };

    let invoice = Invoice {
        id: uuid::Uuid::new_v4().to_string(),
        address_index: index,
        address: address.clone(),
        amount: payload.amount,
        amount_raw: amount_raw.into(),
        paid: "0".to_string(),
        paid_raw: U256::from(0),
        token: payload.token,
        network: payload.network.clone(),
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
        status: InvoiceStatus::Pending,
    };

    if let Err(e) = state.db.add_invoice(&invoice).await {
        return format!("Error: failed to add invoice: {}", e);
    }
    if let Err(e) = state.db.add_watch_address(&payload.network, &address).await {
        return format!("Error: failed to add payment address to watch_addresses: {}", e)
    }

    format!("Pay to: {:?} (index {})", address, index)
}

pub async fn get_invoices(
    State(state): State<Arc<AppState>>
) -> Json<Vec<Invoice>> {
    Json(state.db.get_invoices().await.unwrap()) // scary!
}

pub async fn get_invoice_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Invoice>, String> {
    let invoice = state.db.get_invoice(&id).await.unwrap();

    match invoice {
        Some(inv) => Ok(Json(inv)),
        None => Err("invoice not found".to_owned()),
    }
}

pub async fn delete_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> String {
    state.db.remove_invoice(&id).await.unwrap();
    "ok".to_owned()
}