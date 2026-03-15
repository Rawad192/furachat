// Routes modération — kick, ban, mute, audit log

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::auth::middleware::AuthUser;
use crate::db::queries::{moderation, servers};
use crate::error::AppError;
use crate::AppState;

/// Routes modération
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/servers/{id}/kick/{uid}", post(kick_user))
        .route("/api/servers/{id}/ban/{uid}", post(ban_user).delete(unban_user))
        .route("/api/servers/{id}/mute/{uid}", post(mute_user).delete(unmute_user))
        .route("/api/servers/{id}/audit-log", get(get_audit_log))
}

/// Corps de requête pour kick/ban/mute
#[derive(Debug, Deserialize)]
struct ModerationRequest {
    reason: Option<String>,
    duration_seconds: Option<i64>,
}

/// Paramètres de requête pour l'audit log
#[derive(Debug, Deserialize)]
struct AuditLogQuery {
    limit: Option<i64>,
    before: Option<String>,
}

/// POST /api/servers/:id/kick/:uid — expulse un membre
async fn kick_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((server_id, user_id)): Path<(String, String)>,
    Json(body): Json<ModerationRequest>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Permission insuffisante".to_string()));
    }

    let reason = body.reason.as_deref().unwrap_or("");
    moderation::kick_user(&conn, &server_id, &user_id, &auth.user_id, reason)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/servers/:id/ban/:uid — bannit un membre
async fn ban_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((server_id, user_id)): Path<(String, String)>,
    Json(body): Json<ModerationRequest>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Permission insuffisante".to_string()));
    }

    let reason = body.reason.as_deref().unwrap_or("");
    moderation::ban_user(
        &conn,
        &server_id,
        &user_id,
        &auth.user_id,
        reason,
        body.duration_seconds,
    )?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/servers/:id/ban/:uid — débannit un membre
async fn unban_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((server_id, user_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Permission insuffisante".to_string()));
    }

    moderation::unban_user(&conn, &server_id, &user_id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/servers/:id/mute/:uid — mute un membre
async fn mute_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((server_id, user_id)): Path<(String, String)>,
    Json(body): Json<ModerationRequest>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Permission insuffisante".to_string()));
    }

    let reason = body.reason.as_deref().unwrap_or("");
    moderation::mute_user(
        &conn,
        &server_id,
        &user_id,
        &auth.user_id,
        reason,
        body.duration_seconds,
    )?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/servers/:id/mute/:uid — unmute un membre
async fn unmute_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((server_id, user_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Permission insuffisante".to_string()));
    }

    moderation::unmute_user(&conn, &server_id, &user_id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/servers/:id/audit-log — consulte le journal d'audit
async fn get_audit_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<Vec<moderation::AuditLogEntry>>, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Permission insuffisante".to_string()));
    }

    let limit = query.limit.unwrap_or(50);
    let entries = moderation::get_audit_log(&conn, &server_id, limit, query.before.as_deref())?;

    Ok(Json(entries))
}
