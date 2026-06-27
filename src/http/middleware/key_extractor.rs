use axum::http::{Request, header::AUTHORIZATION};
use tower_governor::errors::GovernorError;
use tower_governor::key_extractor::KeyExtractor;

#[derive(Clone)]
pub struct ApiKeyExtractor;

impl KeyExtractor for ApiKeyExtractor {
    type Key = String;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        let headers = req.headers();

        if let Some(auth_header) = headers.get(AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                let cleaned = auth_str.replace("Bearer ", "");
                return Ok(cleaned);
            }
        }

        if let Some(query) = req.uri().query() {
            for param in query.split('&') {
                let mut parts = param.split('=');
                if let (Some(key), Some(val)) = (parts.next(), parts.next()) {
                    if key == "api_key" {
                        return Ok(val.to_string());
                    }
                }
            }
        }

        Err(GovernorError::UnableToExtractKey)
    }
}