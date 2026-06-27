use thiserror::Error;
use crate::db::error::DbError;

#[derive(Debug, Error)]
pub enum VerifyKeyError {
    #[error(transparent)]
    Db(#[from] DbError),

    #[error("Key is disabled")]
    KeyDisabled,

    #[error("Invalid API key")]
    InvalidApiKey,
}