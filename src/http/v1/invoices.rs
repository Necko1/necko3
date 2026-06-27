use crate::auth::middleware::RequireAuth;
use crate::auth::{ReadInvoicesPerm, WriteInvoicesPerm};
use crate::error::ApiError;
use crate::http::extractor::json::JsonArg;
use crate::http::extractor::path::PathArg;
use crate::http::extractor::query::QueryArg;
use crate::models::request::{CreateInvoiceReq, QueryPagination};
use crate::models::response::PaginatedVecPage;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use necko3_core::builder::invoice_config::{ExpirationTime, PaymentAddress, PaymentAsset, PaymentSpec};
use necko3_core::prelude::db::traits::DatabaseExt;
use necko3_core::types::core::db::InvoiceFilter;
use necko3_core::types::core::Invoice;
use std::time::Duration;
use uuid::Uuid;
use crate::models::filter::QueryInvoiceFilter;
use crate::openapi::schemas::{CommonErrors, CreateInvoiceReqSchema, ErrorResponseSchema, InvoiceSchema, PaginatedVecPageSchema, QueryInvoiceFilterParams, QueryPaginationParams};

#[utoipa::path(
    post,
    path = "/v1/invoices",
    tag = "invoices",
    summary = "Create a new invoice",
    description = "Generates a new payment invoice, allocating a unique address on the specified network for the customer to send funds to.\n\n**Requires Permission:** `write_invoices`",
    security(
        ("bearer_auth" = [])
    ),
    request_body(
        content = CreateInvoiceReqSchema,
        description = "Invoice configuration including amount, network, asset, and optional webhooks."
    ),
    responses(
        (status = 200, description = "Invoice created successfully", body = InvoiceSchema),
        (status = 400, description = "Validation error (e.g., zero amount or duration)", body = ErrorResponseSchema, example = json!({
            "error": {
                "type": "invalid_request_error",
                "code": "parameter_invalid",
                "message": "Invoice amount should be above zero",
                "param": "amount"
            }
        })),
        CommonErrors
    )
)]
pub async fn create_invoice<D: DatabaseExt>(
    _auth: RequireAuth<WriteInvoicesPerm>,
    State(state): State<AppState<D>>,
    JsonArg(payload): JsonArg<CreateInvoiceReq>,
) -> Result<(StatusCode, Json<Invoice>), ApiError> {
    if payload.amount <= 0f64 {
        return Err(ApiError::BadRequest {
            message: "Invoice amount should be above zero".to_string(),
            param: "amount".to_string(),
        })
    }

    if payload.duration == 0 {
        return Err(ApiError::BadRequest {
            message: "Invoice duration should be above zero seconds".to_string(),
            param: "duration".to_string(),
        })
    }

    let payment_asset = PaymentAsset::Unknown(payload.asset);
    let duration = Duration::from_secs(payload.duration);

    let invoice = state.core.create_invoice(
        PaymentSpec::new(payload.network, payload.amount)
            .with_asset(payment_asset),
        PaymentAddress::GenerateNew,
        payload.webhook_config,
        ExpirationTime::Duration(duration)
    ).await?;

    Ok((StatusCode::OK, Json(invoice)))
}

#[utoipa::path(
    get,
    path = "/v1/invoices",
    tag = "invoices",
    summary = "List all invoices",
    description = "Retrieves a paginated list of all created invoices. Can be filtered by status, network, address, or token.\n\n**Requires Permission:** `read_invoices`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        QueryInvoiceFilterParams,
        QueryPaginationParams
    ),
    responses(
        (status = 200, description = "List of invoices retrieved successfully", body = PaginatedVecPageSchema<InvoiceSchema>),
        CommonErrors
    )
)]
pub async fn list_invoices<D: DatabaseExt>(
    _auth: RequireAuth<ReadInvoicesPerm>,
    QueryArg(filter): QueryArg<QueryInvoiceFilter>,
    QueryArg(pagination): QueryArg<QueryPagination>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<PaginatedVecPage<Invoice>>), ApiError> {
    let filter = InvoiceFilter {
        status: filter.status,
        address: filter.address,
        network: filter.network,
        token: filter.token,
        pagination: pagination.into(),
    };

    let invoices = state.core.db().get_invoices(filter).await?;

    Ok((StatusCode::OK, Json(invoices.into())))
}

#[utoipa::path(
    get,
    path = "/v1/invoices/{id}",
    tag = "invoices",
    summary = "Get invoice details",
    description = "Retrieves complete internal details of a specific invoice.\n\n**Requires Permission:** `read_invoices`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "The unique identifier (UUID) of the invoice")
    ),
    responses(
        (status = 200, description = "Invoice details retrieved successfully", body = InvoiceSchema),
        CommonErrors
    )
)]
pub async fn get_invoice<D: DatabaseExt>(
    _auth: RequireAuth<ReadInvoicesPerm>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<Invoice>), ApiError> {
    let invoice = state.core.db().get_invoice(id).await?
        .ok_or(ApiError::NotFound(format!("Invoice '{id}' not found")))?;

    Ok((StatusCode::OK, Json(invoice)))
}

#[utoipa::path(
    post,
    path = "/v1/invoices/{id}/cancel",
    tag = "invoices",
    summary = "Cancel an invoice",
    description = "Manually marks a pending invoice as `Cancelled`. It will no longer accept payments or trigger success webhooks.\n\n**Requires Permission:** `write_invoices`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "The unique identifier (UUID) of the invoice to cancel")
    ),
    responses(
        (status = 200, description = "Invoice cancelled successfully", body = InvoiceSchema),
        CommonErrors
    )
)]
pub async fn cancel_invoice<D: DatabaseExt>(
    _auth: RequireAuth<WriteInvoicesPerm>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<Invoice>), ApiError> {
    let db = state.core.db();

    db.cancel_invoice(id).await?;

    let invoice = db.get_invoice(id).await?
        .ok_or(ApiError::NotFound(format!("Invoice '{id}' not found")))?;

    Ok((StatusCode::OK, Json(invoice)))
}