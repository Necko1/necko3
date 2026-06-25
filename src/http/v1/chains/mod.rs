use crate::auth::middleware::RequireAuth;
use crate::auth::{AnyValidKey, FullAccess};
use crate::error::ApiError;
use crate::http::extractor::json::JsonArg;
use crate::http::extractor::path::PathArg;
use crate::models::response::VecResponse;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use necko3_core::builder::chain_config::ChainConfig;
use necko3_core::prelude::db::DatabaseExt;
use necko3_core::types::core::{ChainData, PartialChainUpdate};

pub mod tokens;

// post /v1/chains
pub async fn add_chain<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    State(state): State<AppState<D>>,
    JsonArg(payload): JsonArg<ChainConfig>,
) -> Result<(StatusCode, Json<ChainData>), ApiError> {
    let chain_data = state.core.add_chain(payload).await?;

    Ok((StatusCode::OK, Json(chain_data)))
}

// get /v1/chains
pub async fn list_chains<D: DatabaseExt>(
    _auth: RequireAuth<AnyValidKey>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<VecResponse<ChainData>>), ApiError> {
    let chains = state.core.db().get_chains().await?;

    Ok((StatusCode::OK, Json(chains.into())))
}

// get /v1/chains/:name
pub async fn get_chain<D: DatabaseExt>(
    _auth: RequireAuth<AnyValidKey>,
    PathArg(name): PathArg<String>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<ChainData>), ApiError> {
    let chain_data = state.core.db().get_chain(&name).await?
        .ok_or(ApiError::NotFound(format!("Chain '{name}' not found")))?;

    Ok((StatusCode::OK, Json(chain_data)))
}

// patch /v1/chains/:name
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

// delete /v1/chains/:name
pub async fn delete_chain<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(name): PathArg<String>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<ChainData>), ApiError> {
    let chain_data = state.core.db().remove_chain(&name).await?;

    Ok((StatusCode::OK, Json(chain_data)))
}
