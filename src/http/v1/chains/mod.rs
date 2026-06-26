use crate::auth::middleware::RequireAuth;
use crate::auth::{AnyValidKey, FullAccess};
use crate::error::ApiError;
use crate::http::extractor::json::JsonArg;
use crate::http::extractor::path::PathArg;
use crate::models::request::CreateChainReq;
use crate::models::response::VecResponse;
use crate::openapi::schemas::{ChainConfigSchema, ChainDataSchema, CommonErrors, PartialChainUpdateSchema, VecResponseSchema};
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use necko3_core::prelude::db::DatabaseExt;
use necko3_core::types::core::{ChainData, PartialChainUpdate};

pub mod tokens;

#[utoipa::path(
    post,
    path = "/v1/chains",
    tag = "chains",
    summary = "Add a new blockchain network",
    description = "Registers a new blockchain network configuration in the system.\n\n**Requires Permission:** `full_access`",
    security(
        ("bearer_auth" = [])
    ),
    request_body(
        content = ChainConfigSchema,
        description = "Configuration details required to initialize the new chain."
    ),
    responses(
        (status = 200, description = "Chain successfully registered", body = ChainDataSchema),
        CommonErrors
    )
)]
pub async fn add_chain<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    State(state): State<AppState<D>>,
    JsonArg(payload): JsonArg<CreateChainReq>,
) -> Result<(StatusCode, Json<ChainData>), ApiError> {
    let chain_data = state.core.add_chain(payload.into()).await?;

    Ok((StatusCode::OK, Json(chain_data)))
}

#[utoipa::path(
    get,
    path = "/v1/chains",
    tag = "chains",
    summary = "List all blockchain networks",
    description = "Retrieves a list of all configured blockchain networks, both active and inactive.\n\n**Requires Permission:** *Any valid API key*",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of blockchain networks retrieved successfully", body = VecResponseSchema<ChainDataSchema>),
        CommonErrors
    )
)]
pub async fn list_chains<D: DatabaseExt>(
    _auth: RequireAuth<AnyValidKey>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<VecResponse<ChainData>>), ApiError> {
    let chains = state.core.db().get_chains().await?;

    Ok((StatusCode::OK, Json(chains.into())))
}

#[utoipa::path(
    get,
    path = "/v1/chains/{name}",
    tag = "chains",
    summary = "Get a specific network",
    description = "Retrieves detailed configuration and state information for a specific blockchain network by its name.\n\n**Requires Permission:** *Any valid API key*",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("name" = String, Path, description = "The unique name of the network (e.g., 'ethereum', 'polygon')")
    ),
    responses(
        (status = 200, description = "Network details retrieved successfully", body = ChainDataSchema),
        CommonErrors
    )
)]
pub async fn get_chain<D: DatabaseExt>(
    _auth: RequireAuth<AnyValidKey>,
    PathArg(name): PathArg<String>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<ChainData>), ApiError> {
    let chain_data = state.core.db().get_chain(&name).await?
        .ok_or(ApiError::NotFound(format!("Chain '{name}' not found")))?;

    Ok((StatusCode::OK, Json(chain_data)))
}

#[utoipa::path(
    patch,
    path = "/v1/chains/{name}",
    tag = "chains",
    summary = "Update network configuration",
    description = "Partially updates the configuration of an existing blockchain network.\n\n**Requires Permission:** `full_access`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("name" = String, Path, description = "The unique name of the network to update")
    ),
    request_body(
        content = PartialChainUpdateSchema,
        description = "Object containing the fields to update. Any omitted fields will remain unchanged."
    ),
    responses(
        (status = 200, description = "Network configuration updated successfully", body = ChainDataSchema),
        CommonErrors
    )
)]
pub async fn update_chain<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(name): PathArg<String>,
    State(state): State<AppState<D>>,
    JsonArg(payload): JsonArg<PartialChainUpdate>,
) -> Result<(StatusCode, Json<ChainData>), ApiError> {
    state.core.db().update_chain_partial(&name, &payload).await?;

    let chain_data = state.core.db().get_chain(&name).await?
        .ok_or(ApiError::NotFound(format!("Chain '{name}' not found")))?;

    Ok((StatusCode::OK, Json(chain_data)))
}

#[utoipa::path(
    delete,
    path = "/v1/chains/{name}",
    tag = "chains",
    summary = "Delete a network",
    description = "Permanently removes a blockchain network configuration from the system.\n\n**Requires Permission:** `full_access`\n\n**Warning:** This operation might be irreversible and could affect associated tokens and historical data depending on database constraints.",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("name" = String, Path, description = "The unique name of the network to delete")
    ),
    responses(
        (status = 200, description = "Network deleted successfully. Returns the last known state of the deleted chain.", body = ChainDataSchema),
        CommonErrors
    )
)]
pub async fn delete_chain<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(name): PathArg<String>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<ChainData>), ApiError> {
    let chain_data = state.core.db().remove_chain(&name).await?;

    Ok((StatusCode::OK, Json(chain_data)))
}
