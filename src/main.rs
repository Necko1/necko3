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

    api::serve(state).await?;

    Ok(())
}