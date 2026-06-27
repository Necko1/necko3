use std::net::SocketAddr;
use std::time::Duration;
use necko3_core::builder::webhook_config::WebhookDispatcherConfig;
use necko3_core::core::NeckoCore;
use necko3_core::prelude::db::backends::{InMemoryAdapter, PostgresAdapter};
use necko3_core::prelude::db::traits::{DatabaseAdapter, DatabaseExt};
use necko3_core::prelude::db::notifying::NotifyingDb;
use necko3_core::types::NeckoEvent;
use tokio::net::TcpListener;
use tower_governor::key_extractor::{KeyExtractor, PeerIpKeyExtractor, SmartIpKeyExtractor};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::models::Permission;
use crate::config::{AppConfig, DatabaseType, RateLimitStrategy};
use crate::db::backends::in_memory::InMemoryAdapter as BackendInMemoryAdapter;
use crate::db::backends::postgres::PostgresAdapter as BackendPostgresAdapter;
use crate::db::DatabaseAdapter as BackendDatabaseAdapter;
use crate::http::{build_router, PrivateRateLimitConf, PublicRateLimitConf};
use crate::http::middleware::cors::cors_from_str;
use crate::models::ApiKeyPrefix;
use crate::state::AppState;

pub mod error;
pub mod config;
pub mod state;
pub mod auth;
pub mod http;
pub mod models;
pub mod db;
pub mod openapi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Configuration initialization error: {}", e);
            std::process::exit(1);
        }
    };

    init_tracing(&config.logging.level, &config.logging.format);

    info!("Initializing application...");

    match config.database.db_type {
        DatabaseType::Postgres => {
            let app_db = BackendPostgresAdapter::new(
                &config.database.url, config.database.max_connections).await?;
            let core_db = PostgresAdapter::new(
                &config.database.url, config.database.max_connections).await?;

            run_app_with_storage(config, app_db, core_db).await
        }

        DatabaseType::Mock => {
            let app_db = BackendInMemoryAdapter::default();
            let core_db = InMemoryAdapter::default();

            run_app_with_storage(config, app_db, core_db).await
        }
    }
}

async fn run_app_with_storage<A, C>(config: AppConfig, app_db: A, core_db: C) -> anyhow::Result<()>
where
    A: BackendDatabaseAdapter + 'static,
    C: DatabaseExt + 'static,
{
    let webhook_config = WebhookDispatcherConfig {
        interval: Duration::from_secs(config.core.webhook_dispatch_interval_secs),
        webhooks_selection_limit: config.core.webhook_selection_limit,
        concurrency_limit: config.core.webhook_concurrency_limit,
    };

    let core = NeckoCore::builder()
        .with_event_type::<NeckoEvent>()
        .with_storage(core_db)
        .with_event_channel_buffer(10000)
        .enable_janitor_service(config.core.enable_janitor)
        .enable_webhook_dispatcher(config.core.enable_webhook_dispatcher)
        .with_janitor_interval(Duration::from_secs(config.core.janitor_interval_secs))
        .with_webhook_config(webhook_config)
        .build().await;

    let state = AppState::new(app_db, core);

    if state.app_db.get_active_keys_amount().await? == 0 {
        info!("No active API keys found. Creating new one");

        let (raw_key, _record) = state.key_store.create_key(
            "Init admin key",
            ApiKeyPrefix::SkLive,
            vec![Permission::FullAccess]
        ).await?;

        println!("New admin key created!: '{}'. Keep it secure.", raw_key);
    }

    match config.rate_limits.public_strategy {
        RateLimitStrategy::Peer => run_app_with_key_extractor(config, state, PeerIpKeyExtractor).await,
        RateLimitStrategy::Smart => run_app_with_key_extractor(config, state, SmartIpKeyExtractor).await
    }
}

async fn run_app_with_key_extractor<K, C>(
    config: AppConfig,
    state: AppState<NotifyingDb<C>>,
    key_extractor: K,
) -> anyhow::Result<()>
where
    K: KeyExtractor + Send + Sync + 'static,
    <K as KeyExtractor>::Key: Send + Sync,

    C: DatabaseExt + 'static,
{
    let cors_layer = cors_from_str(&config.server.cors_allowed_origins);

    let pub_rlc = PublicRateLimitConf {
        replenish_interval_millis: config.rate_limits.public_replenish_interval_millis,
        burst_size: config.rate_limits.public_burst_size,
        key_extractor,
    };

    let priv_rlc = PrivateRateLimitConf {
        replenish_interval_millis: config.rate_limits.private_replenish_interval_millis,
        burst_size: config.rate_limits.private_burst_size,
    };

    let app = build_router(
        state,
        pub_rlc, priv_rlc,
        cors_layer,
        config.server.include_swagger
    );

    info!("Starting HTTP server on {}", config.server.bind_address);
    let listener = TcpListener::bind(&config.server.bind_address).await
        .map_err(|e| {
            error!(address = %config.server.bind_address, error = %e, "Failed to bind TCP listener");
            e
        })?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    ).await?;

    Ok(())
}

fn init_tracing(level: &str, format: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    let fmt_layer = tracing_subscriber::fmt::layer();

    match format {
        "compact" => {
            tracing_subscriber::registry()
                .with(fmt_layer.compact().with_target(false))
                .with(filter)
                .init();
        }
        "json" => {
            tracing_subscriber::registry()
                .with(fmt_layer.json())
                .with(filter)
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(fmt_layer)
                .with(filter)
                .init();
        }
    };
}