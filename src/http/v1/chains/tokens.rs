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
use necko3_core::builder::token_config::TokenConfig;
use necko3_core::prelude::db::traits::DatabaseExt;
use necko3_core::types::core::TokenData;
use crate::openapi::schemas::{CommonErrors, TokenConfigSchema, TokenDataSchema, VecResponseSchema};

#[utoipa::path(
    post,
    path = "/v1/chains/{name}/tokens",
    tag = "chains",
    summary = "Add a new token to a network",
    description = "Registers a new token (e.g., ERC20) to an existing blockchain network.\n\n**Requires Permission:** `full_access`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("name" = String, Path, description = "The canonical name of the network (e.g., 'ethereum')")
    ),
    request_body(
        content = TokenConfigSchema,
        description = "Configuration details for the new token."
    ),
    responses(
        (status = 200, description = "Token successfully registered", body = TokenDataSchema),
        CommonErrors
    )
)]
pub async fn add_token<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(name): PathArg<String>,
    State(state): State<AppState<D>>,
    JsonArg(payload): JsonArg<TokenConfig>,
) -> Result<(StatusCode, Json<TokenData>), ApiError> {
    let token_data = state.core.add_token(name, payload).await?;

    Ok((StatusCode::OK, Json(token_data)))
}

#[utoipa::path(
    get,
    path = "/v1/chains/{name}/tokens",
    tag = "chains",
    summary = "List all tokens for a network",
    description = "Retrieves a list of all supported tokens registered under a specific blockchain network.\n\n**Requires Permission:** *Any valid API key*",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("name" = String, Path, description = "The canonical name of the network")
    ),
    responses(
        (status = 200, description = "List of tokens retrieved successfully", body = VecResponseSchema<TokenDataSchema>),
        CommonErrors
    )
)]
pub async fn list_tokens<D: DatabaseExt>(
    _auth: RequireAuth<AnyValidKey>,
    PathArg(name): PathArg<String>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<VecResponse<TokenData>>), ApiError> {
    let tokens = state.core.db().get_tokens(&name).await?;

    Ok((StatusCode::OK, Json(tokens.into())))
}

#[utoipa::path(
    get,
    path = "/v1/chains/{name}/tokens/{sym}",
    tag = "chains",
    summary = "Get specific token details",
    description = "Retrieves detailed configuration and state information for a specific token on a given network.\n\n**Requires Permission:** *Any valid API key*",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("name" = String, Path, description = "The canonical name of the network"),
        ("sym" = String, Path, description = "The ticker symbol of the token (e.g., 'USDT')")
    ),
    responses(
        (status = 200, description = "Token details retrieved successfully", body = TokenDataSchema),
        CommonErrors
    )
)]
pub async fn get_token<D: DatabaseExt>(
    _auth: RequireAuth<AnyValidKey>,
    PathArg((name, sym)): PathArg<(String, String)>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<TokenData>), ApiError> {
    let db = state.core.db();
    if !db.chain_exists(&name).await? {
        return Err(ApiError::NotFound(format!("Chain '{name}' not found")))
    }

    let token_data = db.get_token(&name, &sym).await?
        .ok_or(ApiError::NotFound(format!("Token '{sym}' not found")))?;

    Ok((StatusCode::OK, Json(token_data)))
}

#[utoipa::path(
    delete,
    path = "/v1/chains/{name}/tokens/{sym}",
    tag = "chains",
    summary = "Delete a token",
    description = "Permanently removes a token configuration from the specified network.\n\n**Requires Permission:** `full_access`",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("name" = String, Path, description = "The canonical name of the network"),
        ("sym" = String, Path, description = "The ticker symbol of the token to delete")
    ),
    responses(
        (status = 200, description = "Token deleted successfully. Returns the last known state of the token.", body = TokenDataSchema),
        CommonErrors
    )
)]
pub async fn delete_token<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg((name, sym)): PathArg<(String, String)>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<TokenData>), ApiError> {
    let chain_data = state.core.db().remove_token(&name, &sym).await?;

    Ok((StatusCode::OK, Json(chain_data)))
}
