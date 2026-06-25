use crate::auth::middleware::RequireAuth;
use crate::auth::{ApiKeyRecord, FullAccess, Permission};
use crate::error::ApiError;
use crate::http::extractor::path::PathArg;
use crate::models::response::{VecResponse, CreateKeyRes, RevokeKeyRes};
use crate::models::request::CreateKeyReq;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use necko3_core::prelude::db::DatabaseExt;
use uuid::Uuid;
use crate::models::ApiKeyPrefix;

// post /v1/api-keys
pub async fn create_key<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    State(state): State<AppState<D>>,
    Json(payload): Json<CreateKeyReq>,
) -> Result<(StatusCode, Json<CreateKeyRes>), ApiError> {
    let validated_permissions = match payload.prefix {
        ApiKeyPrefix::PkLive => {
            if payload.permissions.iter().any(|p| p != &Permission::PublicRead) {
                return Err(ApiError::BadRequest {
                    message: "Publishable keys (pk_) can only have PublicRead permission".into(),
                    param: "permissions".into(),
                });
            }

            vec![Permission::PublicRead]
        },
        ApiKeyPrefix::SkLive => {
            payload.permissions
        }
    };

    let (raw_key, record) = state.key_store
        .create_key(&payload.name, payload.prefix, validated_permissions)
        .await?;

    let res = CreateKeyRes { raw_key, key_id: record.id };

    Ok((StatusCode::OK, Json(res)))
}

// get /v1/api-keys
pub async fn list_api_keys<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<VecResponse<ApiKeyRecord>>), ApiError> {
    let keys = state.app_db.get_keys().await?;

    Ok((StatusCode::OK, Json(keys.into())))
}

// get /v1/api-keys/:id
pub async fn get_key_record<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<ApiKeyRecord>), ApiError> {
    let record = state.app_db.get_key_by_id(id).await?
        .ok_or(ApiError::NotFound(format!("Api key '{id}' not found")))?;

    Ok((StatusCode::OK, Json(record)))
}

// post /v1/api-keys/:id/revoke
pub async fn revoke_key<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<RevokeKeyRes>), ApiError> {
    let record = state.app_db.deactivate_key(id).await?;

    let res = RevokeKeyRes { record, revoked_at: Utc::now(), };

    Ok((StatusCode::OK, Json(res)))
}