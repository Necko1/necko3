use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::Json;
use axum::response::Response;
use necko3_core::prelude::db::traits::DatabaseExt;
use necko3_core::types::core::db::{PaginatedVec, PaymentFilter};
use uuid::Uuid;
use crate::auth::middleware::RequireAuth;
use crate::auth::PublicReadPerm;
use crate::error::ApiError;
use crate::http::extractor::path::PathArg;
use crate::http::extractor::query::QueryArg;
use crate::http::ws::handle_socket;
use crate::models::request::QueryPagination;
use crate::models::public::{PublicInvoiceModel, PublicPaymentModel};
use crate::models::response::PaginatedVecPage;
use crate::openapi::schemas::{CommonErrors, PaginatedVecPageSchema, PublicInvoiceModelSchema, PublicPaymentModelSchema, QueryPaginationParams};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/v1/checkout/invoice/{id}/ws",
    tag = "checkout",
    summary = "Subscribe to invoice updates (WebSocket)",
    description = "Establishes a WebSocket connection to receive real-time updates about the specified invoice's status and incoming payments.\n\n**Requires Permission:** `public_read`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "The unique identifier (UUID) of the invoice")
    ),
    responses(
        (status = 101, description = "WebSocket protocol successfully upgraded"),
        CommonErrors
    )
)]
pub async fn invoice_ws_handler<D: DatabaseExt + 'static>(
    _auth: RequireAuth<PublicReadPerm>,

    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,

    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, id, state))
}

#[utoipa::path(
    get,
    path = "/v1/checkout/invoice/{id}",
    tag = "checkout",
    summary = "Get public invoice details",
    description = "Retrieves public, safe-to-display details of an invoice for the checkout page.\n\n**Requires Permission:** `public_read`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "The unique identifier (UUID) of the invoice")
    ),
    responses(
        (status = 200, description = "Public invoice details retrieved successfully", body = PublicInvoiceModelSchema),
        CommonErrors
    )
)]
pub async fn get_checkout_invoice<D: DatabaseExt>(
    _auth: RequireAuth<PublicReadPerm>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<PublicInvoiceModel>), ApiError> {
    let invoice = state.core.db().get_invoice(id).await?
        .ok_or(ApiError::NotFound(format!("Invoice '{id}' not found")))?;

    let pub_invoice = invoice.into();

    Ok((StatusCode::OK, Json(pub_invoice)))
}

#[utoipa::path(
    get,
    path = "/v1/checkout/invoice/{id}/payments",
    tag = "checkout",
    summary = "List payments for an invoice",
    description = "Retrieves a paginated list of all payments (pending, confirming, and confirmed) directed to a specific invoice's address.\n\n**Requires Permission:** `public_read`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "The unique identifier (UUID) of the invoice"),
        QueryPaginationParams
    ),
    responses(
        (status = 200, description = "List of payments retrieved successfully", body = PaginatedVecPageSchema<PublicPaymentModelSchema>),
        CommonErrors
    )
)]
pub async fn get_checkout_invoice_payments<D: DatabaseExt>(
    _auth: RequireAuth<PublicReadPerm>,
    PathArg(id): PathArg<Uuid>,
    QueryArg(pagination): QueryArg<QueryPagination>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<PaginatedVecPage<PublicPaymentModel>>), ApiError> {
    let invoice = state.core.db().get_invoice(id).await?
        .ok_or(ApiError::NotFound(format!("Invoice '{id}' not found")))?;

    let payments = state.core.db().get_payments(PaymentFilter {
        to: Some(invoice.address),
        network: Some(invoice.network),
        token: Some(invoice.token),
        pagination: pagination.into(),
        ..Default::default()
    }).await?;

    let (total, offset, limit) = (payments.total, payments.offset, payments.limit);

    let mut public_payments = vec![];

    for payment in payments {
        let amount_human = state.core.parse_u256_as_string(
            &payment.network, payment.amount_raw, invoice.decimals)?;
        let public_payment = PublicPaymentModel::from(payment, amount_human);

        public_payments.push(public_payment);
    }

    let paginated_vec = PaginatedVec::new(public_payments, total, offset, limit);

    Ok((StatusCode::OK, Json(paginated_vec.into())))
}