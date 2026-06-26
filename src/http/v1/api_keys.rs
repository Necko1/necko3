use crate::auth::middleware::RequireAuth;
use crate::auth::FullAccess;
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
use crate::models::{ApiKeyPrefix, ApiKeyRecord, Permission};
use crate::openapi::schemas::{ApiKeyRecordSchema, CommonErrors, CreateKeyReqSchema, CreateKeyResSchema, RevokeKeyResSchema, VecResponseSchema};

#[utoipa::path(
    post,
    path = "/v1/api-keys",
    tag = "auth",
    summary = "Create a new API key",
    description = "Generates a new API key (publishable or secret) with specific permissions.\n\n**Requires Permission:** `full_access`",
    security(
        ("bearer_auth" = [])
    ),
    request_body(
        content = CreateKeyReqSchema,
        description = "Details for the new API key. Note that `pk_live` keys are strictly limited to `public_read` permission."
    ),
    responses(
        (status = 200, description = "API key created successfully", body = CreateKeyResSchema),
        CommonErrors
    )
)]
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

#[utoipa::path(
    get,
    path = "/v1/api-keys",
    tag = "auth",
    summary = "List all API keys",
    description = "Retrieves a list of all generated API keys (metadata only, secrets are never returned).\n\n**Requires Permission:** `full_access`",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of API keys retrieved successfully", body = VecResponseSchema<ApiKeyRecordSchema>),
        CommonErrors
    )
)]
pub async fn list_api_keys<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<VecResponse<ApiKeyRecord>>), ApiError> {
    let keys = state.app_db.get_keys().await?;

    Ok((StatusCode::OK, Json(keys.into())))
}

#[utoipa::path(
    get,
    path = "/v1/api-keys/{id}",
    tag = "auth",
    summary = "Get API key details",
    description = "Retrieves metadata for a specific API key by its UUID.\n\n**Requires Permission:** `full_access`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "The unique identifier (UUID) of the API key record")
    ),
    responses(
        (status = 200, description = "API key metadata retrieved successfully", body = ApiKeyRecordSchema),
        CommonErrors
    )
)]
pub async fn get_key_record<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<ApiKeyRecord>), ApiError> {
    let record = state.app_db.get_key_by_id(id).await?
        .ok_or(ApiError::NotFound(format!("Api key '{id}' not found")))?;

    Ok((StatusCode::OK, Json(record)))
}

#[utoipa::path(
    post,
    path = "/v1/api-keys/{id}/revoke",
    tag = "auth",
    summary = "Revoke an API key",
    description = "Deactivates an API key immediately, preventing any further requests using it. This action cannot be easily reversed via the API.\n\n**Requires Permission:** `full_access`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "The unique identifier (UUID) of the API key to revoke")
    ),
    responses(
        (status = 200, description = "API key successfully revoked", body = RevokeKeyResSchema),
        CommonErrors
    )
)]
pub async fn revoke_key<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(id): PathArg<Uuid>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<RevokeKeyRes>), ApiError> {
    let record = state.app_db.deactivate_key(id).await?;

    let res = RevokeKeyRes { record, revoked_at: Utc::now(), };

    Ok((StatusCode::OK, Json(res)))
}