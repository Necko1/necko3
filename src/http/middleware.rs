pub mod key_extractor;
pub mod cors;

use axum::response::{IntoResponse, Response};
use governor::middleware::StateInformationMiddleware;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::GovernorError;
use tower_governor::key_extractor::KeyExtractor;
use crate::error::ApiError;
use crate::http::middleware::key_extractor::ApiKeyExtractor;

pub fn get_governor_conf<K: KeyExtractor>(
    replenish_interval_millis: u64,
    burst_size: u32,
    key_extractor: K
) -> GovernorConfig<K, StateInformationMiddleware> {
    let replenish_interval_millis = replenish_interval_millis.max(1);
    let burst_size = burst_size.max(1);

    GovernorConfigBuilder::default()
        .per_millisecond(replenish_interval_millis)
        .burst_size(burst_size)
        .use_headers()
        .key_extractor(key_extractor)
        .finish()
        .unwrap()
}

pub fn get_api_key_governor_conf(
    replenish_interval_millis: u64,
    burst_size: u32,
) -> GovernorConfig<ApiKeyExtractor, StateInformationMiddleware> {
    get_governor_conf(replenish_interval_millis, burst_size, ApiKeyExtractor)
}

pub fn handle_governor_error(
    error: GovernorError
) -> Response {
    match error {
        GovernorError::UnableToExtractKey => {
            ApiError::ApiKeyMissing.into_response()
        }
        GovernorError::TooManyRequests { .. } => {
            ApiError::TooManyRequests.into_response()
        }
        other => {
            other.into_response().map(axum::body::Body::from)
        }
    }
}