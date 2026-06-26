use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum::{AsRefStr, Display, EnumString};

pub mod public;
pub mod response;
pub mod request;
pub mod filter;

#[derive(Clone, Copy, Debug, Serialize, Deserialize,
    Display, EnumString, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ApiKeyPrefix {
    SkLive,
    PkLive,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKeyRecord {
    pub id: uuid::Uuid,
    pub name: String,
    pub prefix: String,

    #[sqlx(json)]
    pub permissions: Vec<Permission>,

    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash,
    Display, EnumString, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    FullAccess,

    WriteInvoices,
    ReadInvoices,
    PublicRead,
}