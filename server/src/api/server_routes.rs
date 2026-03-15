// Routes serveur — CRUD serveurs, gestion des membres

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use std::path::PathBuf;

use crate::auth::middleware::AuthUser;
use crate::db::queries::servers;
use crate::error::AppError;
use crate::models::server::{CreateServerRequest, ServerDetails, UpdateServerRequest};
use crate::AppState;

/// Routes serveur
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/servers", post(create_server))
        .route("/api/servers/{id}", get(get_server).patch(update_server).delete(delete_server))
        .route("/api/servers/{id}/icon", post(upload_icon))
        .route("/api/servers/{id}/banner", post(upload_banner))
        .route("/api/servers/{id}/leave", post(leave_server))
}

/// POST /api/servers — crée un nouveau serveur
async fn create_server(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateServerRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("Le nom du serveur ne peut pas être vide".to_string()));
    }

    let conn = state.pool.get()?;
    let server = servers::create_server(&conn, &body.name, &auth.user_id)?;

    Ok(Json(serde_json::json!({ "server": server })))
}

/// GET /api/servers/:id — récupère les détails d'un serveur
async fn get_server(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
) -> Result<Json<ServerDetails>, AppError> {
    let conn = state.pool.get()?;

    // Vérifie que l'utilisateur est membre
    if !servers::is_member(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Vous n'êtes pas membre de ce serveur".to_string()));
    }

    let details = servers::get_server_details(&conn, &server_id)?;
    Ok(Json(details))
}

/// PATCH /api/servers/:id — modifie un serveur
async fn update_server(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
    Json(body): Json<UpdateServerRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut modifier le serveur".to_string()));
    }

    let server = servers::update_server(&conn, &server_id, body.name.as_deref())?;
    Ok(Json(serde_json::json!({ "server": server })))
}

/// DELETE /api/servers/:id — supprime un serveur
async fn delete_server(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut supprimer le serveur".to_string()));
    }

    servers::delete_server(&conn, &server_id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/servers/:id/icon — upload de l'icône
async fn upload_icon(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.pool.get()?;
    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut modifier l'icône".to_string()));
    }

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if field.name() == Some("image") || field.name() == Some("file") {
            let filename = format!("{}.png", server_id);
            let path = PathBuf::from(&state.config.data_dir).join("avatars").join(&filename);
            let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
            tokio::fs::write(&path, &data).await?;

            let icon_path = format!("avatars/{}", filename);
            servers::update_server_icon(&conn, &server_id, &icon_path)?;

            return Ok(Json(serde_json::json!({ "icon_path": icon_path })));
        }
    }

    Err(AppError::BadRequest("Aucun fichier fourni".to_string()))
}

/// POST /api/servers/:id/banner — upload de la bannière
async fn upload_banner(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.pool.get()?;
    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut modifier la bannière".to_string()));
    }

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if field.name() == Some("image") || field.name() == Some("file") {
            let filename = format!("{}_banner.png", server_id);
            let path = PathBuf::from(&state.config.data_dir).join("banners").join(&filename);
            let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
            tokio::fs::write(&path, &data).await?;

            let banner_path = format!("banners/{}", filename);
            servers::update_server_banner(&conn, &server_id, &banner_path)?;

            return Ok(Json(serde_json::json!({ "banner_path": banner_path })));
        }
    }

    Err(AppError::BadRequest("Aucun fichier fourni".to_string()))
}

/// POST /api/servers/:id/leave — quitter un serveur
async fn leave_server(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    if servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::BadRequest("Le propriétaire ne peut pas quitter son serveur (supprimez-le à la place)".to_string()));
    }

    servers::remove_member(&conn, &server_id, &auth.user_id)?;
    Ok(StatusCode::NO_CONTENT)
}
