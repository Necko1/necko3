use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use necko3_core::builder::invoice_config::error::InvoiceCreationError;
use necko3_core::error::UnitConversionError;
use necko3_core::prelude::db::DbError;
use crate::db::error::DbError as BackendDbError;
use serde::Serialize;
use tracing::error;

#[derive(Serialize)]
pub struct ErrorObject {
    #[serde(rename = "type")]
    pub err_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorObject,
}

pub enum ApiError {
    NotFound(String),
    BadRequest { message: String, param: String },
    Internal(Box<dyn std::error::Error + Send + Sync>),

    // auth middleware
    ApiKeyMissing,
    ApiKeyInvalid,
    ApiKeyDisabled,
    InsufficientPermissions,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, err_obj) = match self {
            ApiError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorObject {
                    err_type: "invalid_request_error".to_string(),
                    code: Some("resource_missing".to_string()),
                    message: msg,
                    param: None
                }
            ),
            ApiError::BadRequest { message, param } => (
                StatusCode::BAD_REQUEST,
                ErrorObject {
                    err_type: "invalid_request_error".to_string(),
                    code: Some("parameter_invalid".to_string()),
                    message,
                    param: Some(param)
                }
            ),
            ApiError::Internal(e) => {
                error!(error = ?e, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorObject {
                        err_type: "api_error".to_string(),
                        code: None,
                        message: "An internal server error occurred.".to_string(),
                        param: None
                    }
                )
            },
            ApiError::ApiKeyMissing => (
                StatusCode::UNAUTHORIZED,
                ErrorObject {
                    err_type: "invalid_request_error".to_string(),
                    code: Some("api_key_missing".to_string()),
                    message: "You did not provide an API key.".to_string(),
                    param: None,
                }
            ),
            ApiError::ApiKeyInvalid =>  (
                StatusCode::UNAUTHORIZED,
                ErrorObject {
                    err_type: "invalid_request_error".to_string(),
                    code: Some("api_key_invalid".to_string()),
                    message: "The provided API key is invalid.".to_string(),
                    param: None,
                }
            ),
            ApiError::ApiKeyDisabled =>  (
                StatusCode::FORBIDDEN,
                ErrorObject {
                    err_type: "invalid_request_error".to_string(),
                    code: Some("api_key_disabled".to_string()),
                    message: "The provided API key has been disabled.".to_string(),
                    param: None,
                }
            ),
            ApiError::InsufficientPermissions =>  (
                StatusCode::FORBIDDEN,
                ErrorObject {
                    err_type: "invalid_request_error".to_string(),
                    code: Some("permission_denied".to_string()),
                    message: "You do not have the required permissions to access this resource.".to_string(),
                    param: None,
                }
            ),
        };

        (status, Json(ErrorResponse { error: err_obj })).into_response()
    }
}

impl From<DbError> for ApiError {
    fn from(value: DbError) -> Self {
        match value {
            DbError::Sqlx(e) => ApiError::Internal(e.into()),
            DbError::Migration(_) => unreachable!(),
            DbError::NotFound { entity, id } =>
                ApiError::NotFound(format!("{entity} '{id}' not found")),
            DbError::DataCorruption(e) =>
                ApiError::Internal(e.into())
        }
    }
}

impl From<BackendDbError> for ApiError {
    fn from(value: BackendDbError) -> Self {
        match value {
            BackendDbError::Sqlx(e) => ApiError::Internal(e.into()),
            BackendDbError::Migration(_) => unreachable!(),
            BackendDbError::NotFound { entity, id } =>
                ApiError::NotFound(format!("{entity} '{id}' not found")),
        }
    }
}

impl From<UnitConversionError> for ApiError {
    fn from(value: UnitConversionError) -> Self {
        ApiError::Internal(value.into())
    }
}

impl From<InvoiceCreationError> for ApiError {
    fn from(value: InvoiceCreationError) -> Self {
        match value {
            InvoiceCreationError::Database(e) => ApiError::Internal(e.into()),
            InvoiceCreationError::Validation(msg) => ApiError::BadRequest {
                message: msg,
                param: "body".to_string(),
            },
            InvoiceCreationError::NetworkNotFound(network) => ApiError::BadRequest {
                message: format!("Specified network ('{}') does not exist",
                                 network),
                param: "network".to_string(),
            },
            InvoiceCreationError::TokenNotFound { symbol, network } => ApiError::BadRequest {
                message: format!("Specified asset ('{}') does not exist on network '{}'",
                                 symbol, network),
                param: "asset".to_string(),
            },
            InvoiceCreationError::WorkerNotInitialized(network) => ApiError::Internal(
                format!("Worker for network '{network}' is not initialized").into()),
            InvoiceCreationError::MissingXpub(network) => ApiError::Internal(
                format!("Missing XPUB for network '{network}'. Cannot generate new addresses").into()),
            InvoiceCreationError::UnitConversion(e) => ApiError::BadRequest {
                message: format!("Failed to parse amount: {}", e),
                param: "asset".to_string(),
            },
            InvoiceCreationError::Adapter(e) => ApiError::Internal(e.into()),
        }
    }
}