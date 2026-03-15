// Routes badges — création, attribution, révocation

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{delete, post},
    Json, Router,
};
use std::path::PathBuf;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::db::queries::{servers, stickers};
use crate::error::AppError;
use crate::AppState;

/// Routes badges
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/servers/{id}/badges", post(create_badge))
        .route("/api/badges/{id}/award/{uid}", post(award_badge).delete(revoke_badge))
}

/// POST /api/servers/:id/badges — crée un badge (multipart: name + icon)
async fn create_badge(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<stickers::Badge>, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut créer des badges".to_string()));
    }

    let mut name = String::new();
    let mut file_data = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        match field.name() {
            Some("name") => {
                name = field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
            }
            Some("icon") | Some("file") => {
                file_data = Some(field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return Err(AppError::BadRequest("Le nom du badge est requis".to_string()));
    }
    let data = file_data.ok_or_else(|| AppError::BadRequest("L'icône du badge est requise".to_string()))?;

    let filename = format!("{}.png", Uuid::new_v4());
    let path = PathBuf::from(&state.config.data_dir).join("badges").join(&filename);
    tokio::fs::write(&path, &data).await?;

    let icon_path = format!("badges/{}", filename);
    let badge = stickers::create_badge(&conn, &server_id, &name, &icon_path)?;

    Ok(Json(badge))
}

/// POST /api/badges/:id/award/:uid — attribue un badge
async fn award_badge(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((badge_id, user_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    let server_id = stickers::get_badge_server_id(&conn, &badge_id)?;
    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Permission insuffisante".to_string()));
    }

    stickers::award_badge(&conn, &badge_id, &user_id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/badges/:id/award/:uid — révoque un badge
async fn revoke_badge(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((badge_id, user_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    let server_id = stickers::get_badge_server_id(&conn, &badge_id)?;
    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Permission insuffisante".to_string()));
    }

    stickers::revoke_badge(&conn, &badge_id, &user_id)?;
    Ok(StatusCode::NO_CONTENT)
}
