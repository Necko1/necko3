use crate::auth::middleware::RequireAuth;
use crate::auth::ReadInvoicesPerm;
use crate::error::ApiError;
use crate::http::extractor::path::PathArg;
use crate::http::extractor::query::QueryArg;
use crate::models::filter::QueryWebhookFilter;
use crate::models::request::QueryPagination;
use crate::models::response::PaginatedVecPage;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use necko3_core::prelude::db::DatabaseExt;
use necko3_core::types::core::db::WebhookFilter;
use necko3_core::types::core::Webhook;
use uuid::Uuid;

// get /v1/webhooks
pub async fn list_webhooks<D: DatabaseExt>(
    _auth: RequireAuth<ReadInvoicesPerm>,
    QueryArg(filter): QueryArg<QueryWebhookFilter>,
    QueryArg(pagination): QueryArg<QueryPagination>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<PaginatedVecPage<Webhook>>), ApiError> {
    let filter = WebhookFilter {
        invoice_id: filter.invoice_id,
        event_type: filter.event_type,
        url: filter.url,
        status: filter.status,
        pagination: pagination.into(),
    };

    let webhooks = state.core.db().get_webhooks(filter).await?;

    Ok((StatusCode::OK, Json(webhooks.into())))
}

// get /v1/webhooks/:id
pub async fn get_webhook<D: DatabaseExt>(
    _auth: RequireAuth<ReadInvoicesPerm>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<Webhook>), ApiError> {
    let webhook = state.core.db().get_webhook(id).await?
        .ok_or(ApiError::NotFound(format!("Webhook '{id}' not found")))?;

    Ok((StatusCode::OK, Json(webhook)))
}