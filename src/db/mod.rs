use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::db::error::DbResult;
use crate::auth::{ApiKeyRecord, Permission};

pub mod backends;
pub mod error;

#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    async fn new(database_url: &str, max_connections: u32) -> DbResult<Self> where Self: Sized;

    async fn get_key_by_id(&self, key_id: Uuid) -> DbResult<Option<ApiKeyRecord>>;
    async fn get_keys(&self) -> DbResult<Vec<ApiKeyRecord>>;
    async fn insert_key(&self, key_id: Uuid, name: &str, key_hash: String, prefix: &str,
                        permissions: Vec<Permission>, is_active: bool, created_at: DateTime<Utc>) -> DbResult<()>;
    async fn get_key_by_hash(&self, key_hash: &str) -> DbResult<Option<ApiKeyRecord>>;
    async fn deactivate_key(&self, key_id: Uuid) -> DbResult<ApiKeyRecord>;
    async fn get_active_keys_amount(&self) -> DbResult<usize>;
}