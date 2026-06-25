use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum::{AsRefStr, Display, EnumString};

pub mod middleware;
pub mod keystore;
pub mod error;

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

pub trait AuthRequirement: Send + Sync + 'static {
    fn required_permission() -> Option<Permission>;
}

pub struct FullAccess;
impl AuthRequirement for FullAccess {
    fn required_permission() -> Option<Permission> { Some(Permission::FullAccess) }
}

pub struct WriteInvoicesPerm;
impl AuthRequirement for WriteInvoicesPerm {
    fn required_permission() -> Option<Permission> { Some(Permission::WriteInvoices) }
}

pub struct ReadInvoicesPerm;
impl AuthRequirement for ReadInvoicesPerm {
    fn required_permission() -> Option<Permission> { Some(Permission::ReadInvoices) }
}

pub struct PublicReadPerm;
impl AuthRequirement for PublicReadPerm {
    fn required_permission() -> Option<Permission> { Some(Permission::PublicRead) }
}

pub struct AnyValidKey;
impl AuthRequirement for AnyValidKey {
    fn required_permission() -> Option<Permission> { None }
}