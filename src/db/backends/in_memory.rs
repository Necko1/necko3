use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use uuid::Uuid;
use crate::auth::{ApiKeyRecord, Permission};
use crate::db::DatabaseAdapter;
use crate::db::error::{DbError, DbResult};

#[derive(Clone)]
struct InMemoryApiKeyRecord {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub prefix: String,
    pub permissions: Vec<Permission>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<InMemoryApiKeyRecord> for ApiKeyRecord {
    fn from(value: InMemoryApiKeyRecord) -> Self {
        Self {
            id: value.id,
            name: value.name,
            prefix: value.prefix,
            permissions: value.permissions,
            is_active: value.is_active,
            created_at: value.created_at,
        }
    }
}

#[derive(Default)]
pub struct InMemoryAdapter {
    keys: DashMap<Uuid, InMemoryApiKeyRecord>,
}

#[async_trait]
impl DatabaseAdapter for InMemoryAdapter {
    async fn new(_database_url: &str, _max_connections: u32) -> DbResult<Self>
    where
        Self: Sized
    {
        Ok(Default::default())
    }

    async fn get_key_by_id(&self, key_id: Uuid) -> DbResult<Option<ApiKeyRecord>> {
        let record = self.keys.get(&key_id).map(|r| r.value().clone().into());
        Ok(record)
    }

    async fn get_keys(&self) -> DbResult<Vec<ApiKeyRecord>> {
        let mut records: Vec<ApiKeyRecord> = self.keys
            .iter()
            .map(|r| r.value().clone().into())
            .collect();

        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(records)
    }

    async fn insert_key(
        &self,
        key_id: Uuid,
        name: &str,
        key_hash: String,
        prefix: &str,
        permissions: Vec<Permission>,
        is_active: bool,
        created_at: DateTime<Utc>
    ) -> DbResult<()> {
        let record = InMemoryApiKeyRecord {
            id: key_id,
            name: name.to_string(),
            key_hash,
            prefix: prefix.to_string(),
            permissions,
            is_active,
            created_at,
        };

        self.keys.insert(key_id, record);
        Ok(())
    }

    async fn get_key_by_hash(&self, key_hash: &str) -> DbResult<Option<ApiKeyRecord>> {
        let record = self.keys
            .iter()
            .find(|r| r.value().key_hash == key_hash)
            .map(|r| r.value().clone().into());

        Ok(record)
    }

    async fn deactivate_key(&self, key_id: Uuid) -> DbResult<ApiKeyRecord> {
        let mut record_ref = self.keys.get_mut(&key_id)
            .ok_or(DbError::NotFound {
                entity: "Api key",
                id: key_id.to_string(),
            })?;

        record_ref.is_active = false;
        let record = record_ref.value().clone();

        Ok(record.into())
    }

    async fn get_active_keys_amount(&self) -> DbResult<usize> {
        Ok(self.keys.len())
    }
}