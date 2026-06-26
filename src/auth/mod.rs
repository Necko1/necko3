use serde::{Serialize};
use crate::models::Permission;

pub mod middleware;
pub mod keystore;
pub mod error;

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