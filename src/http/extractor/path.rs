use axum::{
    extract::{rejection::PathRejection, FromRequestParts, Path},
    http::request::Parts,
};
use axum::extract::path::{ErrorKind, FailedToDeserializePathParams};
use axum::extract::RawPathParams;
use serde::de::DeserializeOwned;
use crate::error::ApiError;

pub struct PathArg<T>(pub T);

impl<S, T> FromRequestParts<S> for PathArg<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Path::from_request_parts(parts, state).await {
            Ok(Path(value)) => Ok(PathArg(value)),
            Err(rejection) => {
                let raw_params = RawPathParams::from_request_parts(parts, state).await.ok();

                let (message, param_name) = match rejection {
                    PathRejection::FailedToDeserializePathParams(failed_params) => {
                        from_failed_params(raw_params, failed_params)
                    }
                    PathRejection::MissingPathParams(_) => (
                        "Missing path parameters".to_string(),
                        "path".to_string(),
                    ),
                    _ => (
                        "Invalid path parameters".to_string(),
                        "path".to_string()
                    )
                };

                Err(ApiError::BadRequest { message, param: param_name })
            }
        }
    }
}

fn from_failed_params(
    raw_params: Option<RawPathParams>,
    failed_params: FailedToDeserializePathParams
) -> (String, String) {
    match failed_params.into_kind() {
        ErrorKind::WrongNumberOfParameters { got, expected } => (
            format!("Wrong number of path parameters (expected {}, got {})", expected, got),
            "path".to_string(),
        ),
        ErrorKind::ParseErrorAtKey { key, value, expected_type } => (
            format!("Failed to parse parameter '{}' (value: '{}') as {}", key, value, expected_type),
            key,
        ),
        ErrorKind::ParseErrorAtIndex { index, value, expected_type } => {
            let key = raw_params
                .as_ref()
                .and_then(|p| p.iter().nth(index).map(|(k, _)| k.to_string()))
                .unwrap_or_else(|| format!("param_at_index_{}", index));

            (
                format!("Failed to parse parameter '{}' (value: '{}') as {}", key, value, expected_type),
                key,
            )
        },
        ErrorKind::ParseError { value, expected_type } => {
            let key = raw_params
                .as_ref()
                .and_then(|p| p.iter().next().map(|(k, _)| k.to_string()))
                .unwrap_or_else(|| "parameter".to_string());

            (
                format!("Failed to parse parameter '{}' (value: '{}') as {}", key, value, expected_type),
                key,
            )
        }
        ErrorKind::InvalidUtf8InPathParam { key } => (
            format!("Parameter '{}' contains invalid UTF-8", key),
            key,
        ),
        ErrorKind::UnsupportedType { name } => (
            format!("Unsupported path parameter type: {}", name),
            "path".to_string(),
        ),
        ErrorKind::DeserializeError { key, value, message } => (
            format!("Invalid value '{}' for parameter '{}': {}", value, key, message),
            key,
        ),
        ErrorKind::Message(msg) => (
            msg,
            "path".to_string()
        ),
        _ => (
            "Invalid path parameters".to_string(),
            "path".to_string(),
        )
    }
}