use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use necko3_core::prelude::db::DatabaseExt;
use necko3_core::types::core::db::PaymentFilter;
use necko3_core::types::core::Payment;
use uuid::Uuid;
use crate::auth::middleware::RequireAuth;
use crate::auth::ReadInvoicesPerm;
use crate::error::ApiError;
use crate::http::extractor::path::PathArg;
use crate::http::extractor::query::QueryArg;
use crate::models::filter::QueryPaymentFilter;
use crate::models::request::QueryPagination;
use crate::models::response::PaginatedVecPage;
use crate::state::AppState;

// get /v1/payments
pub async fn list_payments<D: DatabaseExt>(
    _auth: RequireAuth<ReadInvoicesPerm>,
    QueryArg(filter): QueryArg<QueryPaymentFilter>,
    QueryArg(pagination): QueryArg<QueryPagination>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<PaginatedVecPage<Payment>>), ApiError> {
    let filter = PaymentFilter {
        from: filter.from,
        to: filter.to,
        network: filter.network,
        token: filter.token,
        block_number: filter.block_number,
        block_hash: filter.block_hash,
        status: filter.status,
        pagination: pagination.into(),
    };
    
    let payments = state.core.db().get_payments(filter).await?;

    Ok((StatusCode::OK, Json(payments.into())))
}

// get /v1/payments/:id
pub async fn get_payment<D: DatabaseExt>(
    _auth: RequireAuth<ReadInvoicesPerm>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<Payment>), ApiError> {
    let payment = state.core.db().get_payment(id).await?
        .ok_or(ApiError::NotFound(format!("Payment '{id}' not found")))?;

    Ok((StatusCode::OK, Json(payment)))
}