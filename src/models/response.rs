use chrono::{DateTime, Utc};
use necko3_core::types::core::db::PaginatedVec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::ApiKeyRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedVecPage<T: Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub page_size: u32,
    pub page: u64,
}

impl<T> From<PaginatedVec<T>> for PaginatedVecPage<T>
where
    T: Serialize,
{
    fn from(value: PaginatedVec<T>) -> Self {
        let page = if value.limit > 0 {
            (value.offset / value.limit as u64) + 1
        } else {
            1
        };

        Self {
            items: value.items,
            total: value.total,
            page_size: value.limit,
            page,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VecResponse<T: Serialize> {
    pub items: Vec<T>,
}

impl<T> From<Vec<T>> for VecResponse<T>
where
    T: Serialize,
{
    fn from(value: Vec<T>) -> Self {
        Self { items: value }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateKeyRes {
    pub raw_key: String,
    pub key_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RevokeKeyRes {
    #[serde(flatten)]
    pub record: ApiKeyRecord,
    pub revoked_at: DateTime<Utc>,
}