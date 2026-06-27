use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::header::CONTENT_TYPE;
use serde::de::DeserializeOwned;
use crate::error::ApiError;

pub struct JsonArg<T>(pub T);

impl<S, T> FromRequest<S> for JsonArg<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let is_json = req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.starts_with("application/json")
                || value.contains("+json"))
            .unwrap_or(false);

        if !is_json {
            return Err(ApiError::BadRequest {
                message: "Expected Content-Type: application/json".to_string(),
                param: "headers".to_string(),
            });
        }

        let bytes = match Bytes::from_request(req, state).await {
            Ok(b) => b,
            Err(err) => {
                return Err(ApiError::BadRequest {
                    message: format!("Failed to read request body: {}", err),
                    param: "body".to_string(),
                });
            }
        };

        let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
        match serde_path_to_error::deserialize(&mut deserializer) {
            Ok(value) => Ok(JsonArg(value)),
            Err(err) => {
                let param_name = err.path().to_string();
                let inner_err = err.into_inner();

                let message = format!(
                    "Failed to parse JSON field '{}': {}",
                    param_name, inner_err
                );

                let final_param = if param_name.is_empty() {
                    "body".to_string()
                } else {
                    param_name
                };

                Err(ApiError::BadRequest { message, param: final_param })
            }
        }
    }
}