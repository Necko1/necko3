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
use necko3_core::prelude::db::DatabaseExt;
use necko3_core::types::core::TokenData;

// post /v1/chains/:name/tokens
pub async fn add_token<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(name): PathArg<String>,
    State(state): State<AppState<D>>,
    JsonArg(payload): JsonArg<TokenConfig>,
) -> Result<(StatusCode, Json<TokenData>), ApiError> {
    let token_data = state.core.add_token(name, payload).await?;

    Ok((StatusCode::OK, Json(token_data)))
}

// get /v1/chains/:name/tokens
pub async fn list_tokens<D: DatabaseExt>(
    _auth: RequireAuth<AnyValidKey>,
    PathArg(name): PathArg<String>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<VecResponse<TokenData>>), ApiError> {
    let tokens = state.core.db().get_tokens(&name).await?;

    Ok((StatusCode::OK, Json(tokens.into())))
}

// get /v1/chains/:name/tokens/:sym
pub async fn get_token<D: DatabaseExt>(
    _auth: RequireAuth<AnyValidKey>,
    PathArg(name): PathArg<String>,
    PathArg(sym): PathArg<String>,
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

// delete /v1/chains/:name/tokens/:sym
pub async fn delete_token<D: DatabaseExt>(
    _auth: RequireAuth<FullAccess>,
    PathArg(name): PathArg<String>,
    PathArg(sym): PathArg<String>,
    State(state): State<AppState<D>>,
) -> Result<(StatusCode, Json<TokenData>), ApiError> {
    let chain_data = state.core.db().remove_token(&name, &sym).await?;

    Ok((StatusCode::OK, Json(chain_data)))
}
