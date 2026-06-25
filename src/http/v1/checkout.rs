use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::Json;
use axum::response::Response;
use necko3_core::prelude::db::DatabaseExt;
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
use crate::state::AppState;

// get /v1/checkout/invoice/:id/ws
pub async fn invoice_ws_handler<D: DatabaseExt + 'static>(
    _auth: RequireAuth<PublicReadPerm>,

    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,

    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, id, state))
}

// get /v1/checkout/invoice/:id
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

// get /v1/checkout/invoice/:id/payments
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