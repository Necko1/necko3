use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::de::DeserializeOwned;
use crate::error::ApiError;

pub struct QueryArg<T>(pub T);

impl<S, T> FromRequestParts<S> for QueryArg<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query_string = parts.uri.query().unwrap_or_default();

        let deserializer = serde_urlencoded::Deserializer::new(
            form_urlencoded::parse(query_string.as_bytes())
        );

        match serde_path_to_error::deserialize(deserializer) {
            Ok(value) => Ok(QueryArg(value)),
            Err(err) => {
                let param_name = err.path().to_string();
                let inner_err = err.into_inner();

                let message = format!(
                    "Failed to parse query parameter '{}': {}",
                    param_name, inner_err
                );

                let final_param = if param_name.is_empty() {
                    "query".to_string()
                } else {
                    param_name
                };

                Err(ApiError::BadRequest { message, param: final_param })
            }
        }
    }
}