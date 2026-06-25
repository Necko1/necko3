use axum::Router;
use axum::routing::{get, post};
use necko3_core::prelude::db::DatabaseExt;
use tower_governor::GovernorLayer;
use tower_governor::key_extractor::KeyExtractor;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use crate::http::middleware::{get_api_key_governor_conf, get_governor_conf};
use crate::state::AppState;

pub mod middleware;
pub mod ws;
pub mod v1;
pub mod extractor;

use v1::*;

pub struct PublicRateLimitConf<K: KeyExtractor> {
    pub replenish_interval_millis: u64,
    pub burst_size: u32,
    pub key_extractor: K
}

pub struct PrivateRateLimitConf {
    pub replenish_interval_millis: u64,
    pub burst_size: u32,
}

pub fn build_router<D, K>(
    state: AppState<D>,
    pub_rlc: PublicRateLimitConf<K>,
    priv_rlc: PrivateRateLimitConf,
    cors_layer: CorsLayer,
) -> Router
where
    D: DatabaseExt + 'static,
    K: KeyExtractor + Send + Sync + 'static,
    <K as KeyExtractor>::Key: Send + Sync,
{
    let pub_governor = get_governor_conf(
        pub_rlc.replenish_interval_millis, pub_rlc.burst_size, pub_rlc.key_extractor);

    let priv_governor = get_api_key_governor_conf(
        priv_rlc.replenish_interval_millis, priv_rlc.burst_size);

    let public_routes = Router::new()
        .route("/checkout/invoice/{id}", get(checkout::get_checkout_invoice))
        .route("/checkout/invoice/{id}/ws", get(checkout::invoice_ws_handler))
        .route("/checkout/invoice/{id}/payments", get(checkout::get_checkout_invoice_payments))

        .layer(GovernorLayer::new(pub_governor));

    let private_routes = Router::new()
        .route("/api-keys", post(api_keys::create_key)
            .get(api_keys::list_api_keys))
        .route("/api-keys/{id}", get(api_keys::get_key_record))
        .route("/api-keys/{id}/revoke", post(api_keys::revoke_key))

        .route("/chains", post(chains::add_chain)
            .get(chains::list_chains))
        .route("/chains/{name}", get(chains::get_chain)
            .patch(chains::update_chain)
            .delete(chains::delete_chain))

        .route("/chains/{name}/tokens", post(chains::tokens::add_token)
            .get(chains::tokens::list_tokens))
        .route("/chains/{name}/tokens/{sym}", get(chains::tokens::get_token)
            .delete(chains::tokens::delete_token))

        .route("/invoices", post(invoices::create_invoice)
            .get(invoices::list_invoices))
        .route("/invoices/{id}", get(invoices::get_invoice))
        .route("/invoices/{id}/cancel", post(invoices::cancel_invoice))

        .route("/payments", get(payments::list_payments))
        .route("/payments/{id}", get(payments::get_payment))

        .route("/webhooks", get(webhooks::list_webhooks))
        .route("/webhooks/{id}", get(webhooks::get_webhook))

        .layer(GovernorLayer::new(priv_governor));

    let v1_routes = public_routes.merge(private_routes);

    Router::new()
        .nest("/v1", v1_routes)

        .with_state(state)
        .layer(cors_layer)
        .layer(TraceLayer::new_for_http())

        .route("/health", get(|| async { "ok" }))
}