use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use moka::future::Cache;
use rand::{rng, Rng};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use crate::auth::{ApiKeyRecord, Permission};
use crate::auth::error::VerifyKeyError;
use crate::db::DatabaseAdapter as BackendDatabaseAdapter;
use crate::db::error::DbResult as BackendDbResult;
use crate::models::ApiKeyPrefix;

pub type RawApiKey = String;

pub struct KeyStore {
    db: Arc<dyn BackendDatabaseAdapter>,
    cache: Cache<[u8; 32], ApiKeyRecord>,
}

impl KeyStore {
    pub fn new(db: Arc<dyn BackendDatabaseAdapter>) -> Self {
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_idle(Duration::from_mins(5))
            .build();

        Self { db, cache }
    }

    fn hash_key(raw_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn create_key(
        &self,
        name: &str,
        prefix: ApiKeyPrefix,
        permissions: Vec<Permission>
    ) -> BackendDbResult<(RawApiKey, ApiKeyRecord)> {
        let mut key_bytes = [0u8; 32];
        rng().fill_bytes(&mut key_bytes);
        let random_str = base_62::encode(&key_bytes);

        let raw_key = format!("{}_{}", prefix, random_str);
        let key_hash = KeyStore::hash_key(&raw_key);
        let key_id = Uuid::new_v4();

        let now = Utc::now();
        
        self.db.insert_key(key_id, name, key_hash, &prefix.to_string(), permissions.clone(), true, now).await?;

        let record = ApiKeyRecord {
            id: key_id,
            name: name.to_string(),
            prefix: prefix.to_string(),
            permissions,
            is_active: true,
            created_at: now,
        };

        Ok((raw_key, record))
    }

    pub async fn verify_key(&self, raw_key: &str) -> Result<ApiKeyRecord, VerifyKeyError> {
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        let raw_hash: [u8; 32] = hasher.finalize().into();

        if let Some(record) = self.cache.get(&raw_hash).await {
            if !record.is_active {
                return Err(VerifyKeyError::KeyDisabled)
            }

            return Ok(record)
        }

        let key_hash = KeyStore::hash_key(raw_key);
        let record_opt = self.db.get_key_by_hash(&key_hash).await?;

        let Some(record) = record_opt else {
            return Err(VerifyKeyError::InvalidApiKey)
        };

        if !record.is_active {
            return Err(VerifyKeyError::KeyDisabled)
        }

        self.cache.insert(raw_hash, record.clone()).await;

        Ok(record)
    }

    pub async fn revoke_key(&self, key_id: Uuid) -> BackendDbResult<()> {
        self.db.deactivate_key(key_id).await?;

        self.cache.invalidate_all();

        Ok(())
    }
}