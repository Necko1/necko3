use axum::http::{header, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn cors_from_str(raw_str: &str) -> CorsLayer {
    let (origin, allow_credentials) = match raw_str.to_lowercase().as_str() {
        "all" | "any" => (AllowOrigin::any(), false),
        _ => {
            let list = raw_str
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<HeaderValue>()
                    .expect("Bad origin"))
                .collect::<Vec<HeaderValue>>();

            (AllowOrigin::list(list), true)
        }
    };

    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::AUTHORIZATION,
        ])
        .allow_credentials(allow_credentials)
}