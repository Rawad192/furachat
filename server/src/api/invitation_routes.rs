// Routes invitations — créer, consulter, rejoindre

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::auth::middleware::AuthUser;
use crate::db::queries::{invitations, servers};
use crate::error::AppError;
use crate::models::invitation::{CreateInviteRequest, InviteInfo, Invitation};
use crate::AppState;

/// Routes invitations
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/servers/{id}/invites", post(create_invite))
        .route("/api/invites/{code}/join", post(join_invite))
        .route("/api/invites/{code}", get(get_invite))
}

/// POST /api/servers/:id/invites — crée une invitation
async fn create_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
    Json(body): Json<CreateInviteRequest>,
) -> Result<Json<Invitation>, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_member(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Vous n'êtes pas membre de ce serveur".to_string()));
    }

    let invite = invitations::create_invitation(
        &conn,
        &server_id,
        &auth.user_id,
        body.expires_in_seconds,
        body.max_uses,
    )?;

    Ok(Json(invite))
}

/// POST /api/invites/:code/join — rejoindre via une invitation
async fn join_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.pool.get()?;

    let server_id = invitations::use_invitation(&conn, &code, &auth.user_id)?;

    // Vérifie que l'utilisateur n'est pas banni
    if crate::db::queries::moderation::is_banned(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Vous êtes banni de ce serveur".to_string()));
    }

    // Ajoute comme membre
    servers::add_member(&conn, &server_id, &auth.user_id)?;

    let server = servers::get_server(&conn, &server_id)?;
    Ok(Json(serde_json::json!({ "server": server })))
}

/// GET /api/invites/:code — infos sur une invitation
async fn get_invite(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<InviteInfo>, AppError> {
    let conn = state.pool.get()?;
    let info = invitations::get_invite_by_code(&conn, &code)?;
    Ok(Json(info))
}
