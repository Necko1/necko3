mod config;
mod model;
mod chain;
mod state;
mod api;
mod db;

use crate::db::mock::MockDatabase;
use crate::db::postgres::Postgres;
use crate::db::Database;
use crate::state::AppState;
use axum::routing::{delete, get, post};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let db_type = std::env::var("DATABASE_TYPE")
        .expect("DATABASE_TYPE must be set");

    let db: Database = match db_type.as_str() {
        "postgres" => {
            let pool = PgPoolOptions::new()
                .max_connections(10)
                .connect(&database_url)
                .await?;

            sqlx::migrate!("./migrations/postgres")
                .run(&pool)
                .await?;

            Database::Postgres(Postgres::init(pool).await?)
        }
        "mock" => Database::Mock(MockDatabase::new()),
        _ => panic!("Unknown DB type")
    };

    let state = AppState::init(db, Duration::from_secs(30));

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))

        .route("/invoice", post(api::create_invoice))
        .route("/invoice", get(api::get_invoices))
        .route("/invoice/{id}", get(api::get_invoice_by_id))
        .route("/invoice/{id}", delete(api::delete_invoice))

        .route("/chain", post(api::add_chain))
        .route("/chain", get(api::get_chains))
        .route("/chain/{name}", get(api::get_chain))
        .route("/chain/{name}", delete(api::remove_chain))

        .route("/chain/{name}/token", post(api::chain::add_token))
        .route("/chain/{name}/token", get(api::chain::get_tokens))
        .route("/chain/{name}/token/{symbol}", get(api::chain::get_token))
        .route("/chain/{name}/token/{symbol}", delete(api::chain::remove_token))
        .with_state(state);

    println!("Started listening on http://127.0.0.1:3000");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}