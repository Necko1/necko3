use axum::extract::Query;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use crate::models::request::ImageProxyParams;
use crate::openapi::schemas::ImageProxyParamsSchema;

#[utoipa::path(
    get,
    path = "/v1/proxy/image",
    tag = "checkout",
    summary = "Proxy and cache external images",
    description = "Proxies external image resources to bypass CORS restrictions (preventing canvas taints). Returns aggressive caching headers to minimize upstream requests.",
    params(
        ImageProxyParamsSchema
    ),
    responses(
        (status = 200, description = "The proxied image bytes.", content_type = "image/png"),
        (status = 400, description = "Failed to fetch the source image or server returned non-success status code."),
        (status = 500, description = "Failed to read or process image bytes on the proxy side.")
    )
)]
pub async fn image_proxy(Query(params): Query<ImageProxyParams>) -> impl IntoResponse {
    let res = match reqwest::get(&params.url).await {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_REQUEST, "Failed to download image").into_response(),
    };

    if !res.status().is_success() {
        return (StatusCode::BAD_REQUEST, "Source server returned an error").into_response();
    }

    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("image/png")
        .to_string();

    let bytes = match res.bytes().await {
        Ok(b) => b,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read image bytes").into_response(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        content_type.parse().unwrap_or(header::HeaderValue::from_static("image/png")),
    );

    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());

    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=31536000, immutable".parse().unwrap(),
    );

    (headers, bytes).into_response()
}