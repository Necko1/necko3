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
use crate::openapi::schemas::{CommonErrors, PaginatedVecPageSchema, QueryPaginationParams, QueryWebhookFilterParams, WebhookSchema};

#[utoipa::path(
    get,
    path = "/v1/webhooks",
    tag = "webhooks",
    summary = "List webhook deliveries",
    description = "Retrieves a paginated list of webhook delivery logs. Useful for auditing and debugging failed event notifications.\n\n**Requires Permission:** `read_invoices`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        QueryWebhookFilterParams,
        QueryPaginationParams
    ),
    responses(
        (status = 200, description = "List of webhook logs retrieved successfully", body = PaginatedVecPageSchema<WebhookSchema>),
        CommonErrors
    )
)]
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

#[utoipa::path(
    get,
    path = "/v1/webhooks/{id}",
    tag = "webhooks",
    summary = "Get webhook delivery details",
    description = "Retrieves the full payload, status, and retry schedule of a specific webhook delivery event.\n\n**Requires Permission:** `read_invoices`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "The unique identifier (UUID) of the webhook delivery log")
    ),
    responses(
        (status = 200, description = "Webhook details retrieved successfully", body = WebhookSchema),
        CommonErrors
    )
)]
pub async fn get_webhook<D: DatabaseExt>(
    _auth: RequireAuth<ReadInvoicesPerm>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<Webhook>), ApiError> {
    let webhook = state.core.db().get_webhook(id).await?
        .ok_or(ApiError::NotFound(format!("Webhook '{id}' not found")))?;

    Ok((StatusCode::OK, Json(webhook)))
}