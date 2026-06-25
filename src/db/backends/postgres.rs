use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use uuid::Uuid;
use crate::auth::{ApiKeyRecord, Permission};
use crate::db::DatabaseAdapter;
use crate::db::error::{DbError, DbResult};

pub struct PostgresAdapter {
    pool: PgPool
}

#[async_trait]
impl DatabaseAdapter for PostgresAdapter {
    async fn new(database_url: &str, max_connections: u32) -> DbResult<Self>
    where
        Self: Sized
    {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations/postgres")
            .dangerous_set_table_name("_sqlx_migrations_back")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }

    async fn get_key_by_id(&self, key_id: Uuid) -> DbResult<Option<ApiKeyRecord>> {
        let record = sqlx::query_as::<_, ApiKeyRecord>(
            "SELECT * FROM api_keys WHERE id = $1"
        )
            .bind(key_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(record)
    }

    async fn get_keys(&self) -> DbResult<Vec<ApiKeyRecord>> {
        let records = sqlx::query_as::<_, ApiKeyRecord>(
            "SELECT * FROM api_keys ORDER BY created_at DESC"
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(records)
    }

    async fn insert_key(&self, key_id: Uuid, name: &str, key_hash: String, prefix: &str, permissions: Vec<Permission>, is_active: bool, created_at: DateTime<Utc>) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO api_keys (id, name, key_hash,
                      prefix, permissions, is_active, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)"#
        )
            .bind(key_id)
            .bind(name)
            .bind(key_hash)
            .bind(prefix)
            .bind(Json(permissions))
            .bind(is_active)
            .bind(created_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_key_by_hash(&self, key_hash: &str) -> DbResult<Option<ApiKeyRecord>> {
        let record = sqlx::query_as::<_, ApiKeyRecord>(
            "SELECT * FROM api_keys WHERE key_hash = $1"
        )
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await?;

        Ok(record)
    }

    async fn deactivate_key(&self, key_id: Uuid) -> DbResult<ApiKeyRecord> {
        let record_opt = sqlx::query_as::<_, ApiKeyRecord>(
            r#"UPDATE api_keys SET is_active = false
                   WHERE id = $1
                   RETURNING *"#
        )
            .bind(key_id)
            .fetch_optional(&self.pool)
            .await?;

        let Some(record) = record_opt else {
            return Err(DbError::NotFound {
                entity: "Api key",
                id: key_id.to_string(),
            })
        };

        Ok(record)
    }

    async fn get_active_keys_amount(&self) -> DbResult<usize> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM api_keys WHERE is_active = true"
        )
            .fetch_one(&self.pool)
            .await?;

        Ok(count as usize)
    }
}