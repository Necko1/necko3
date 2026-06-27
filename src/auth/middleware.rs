use crate::auth::error::VerifyKeyError;
use crate::auth::keystore::KeyStore;
use crate::auth::AuthRequirement;
use crate::models::{ApiKeyRecord, Permission};
use crate::error::ApiError;
use axum::extract::{FromRef, FromRequestParts, Query};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use axum::http::header::AUTHORIZATION;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthQuery {
    pub api_key: Option<String>,
}

pub struct RequireAuth<R: AuthRequirement>(pub ApiKeyRecord, std::marker::PhantomData<R>);

impl<S, R> FromRequestParts<S> for RequireAuth<R>
where
    S: Send + Sync,
    Arc<KeyStore>: FromRef<S>,
    R: AuthRequirement,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let key_store = Arc::<KeyStore>::from_ref(state);

        let auth_header = parts.headers.get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.replace("Bearer ", ""));

        let query_key = match Query::<AuthQuery>::from_request_parts(parts, state).await {
            Ok(query) => query.api_key.clone(),
            Err(_) => None,
        };

        let provided_key = match auth_header.or(query_key) {
            Some(key) => key,
            None => {
                return Err(ApiError::ApiKeyMissing.into_response());
            }
        };

        let record = match key_store.verify_key(&provided_key).await {
            Ok(r) => r,
            Err(err) => {
                let api_error = match err {
                    VerifyKeyError::Db(e) => ApiError::Internal(Box::new(e)),
                    VerifyKeyError::KeyDisabled => ApiError::ApiKeyDisabled,
                    VerifyKeyError::InvalidApiKey => ApiError::ApiKeyInvalid,
                };

                return Err(api_error.into_response());
            }
        };

        if let Some(required_perm) = R::required_permission() {
            let has_permission = record.permissions.contains(&required_perm)
                || record.permissions.contains(&Permission::FullAccess);

            if !has_permission {
                return Err(ApiError::InsufficientPermissions.into_response());
            }
        }

        Ok(RequireAuth(record, std::marker::PhantomData))
    }
}