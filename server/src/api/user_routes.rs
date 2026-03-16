// Routes utilisateur — profil, avatar, bannière

use axum::{
    extract::{Multipart, Path, State},
    routing::{get, patch, post},
    Json, Router,
};
use std::path::PathBuf;

use crate::api::upload_helpers::read_image_field;
use crate::auth::middleware::AuthUser;
use crate::db::queries::users;
use crate::error::AppError;
use crate::models::user::{UpdateProfileRequest, User};
use crate::AppState;

/// Routes utilisateur
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/users/@me", get(get_me).patch(update_me))
        .route("/api/users/@me/avatar", post(upload_avatar))
        .route("/api/users/@me/banner", post(upload_banner))
        .route("/api/users/{id}", get(get_user))
}

/// GET /api/users/@me — récupère le profil de l'utilisateur courant
async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<User>, AppError> {
    let conn = state.pool.get()?;
    let user = users::get_user_by_id(&conn, &auth.user_id)?;
    Ok(Json(user))
}

/// PATCH /api/users/@me — met à jour le profil
async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<User>, AppError> {
    let conn = state.pool.get()?;

    let social_links_str = body.social_links.as_ref().map(|v| v.to_string());

    let user = users::update_user_profile(
        &conn,
        &auth.user_id,
        body.username.as_deref(),
        body.bio.as_deref(),
        body.status_text.as_deref(),
        body.status_emoji.as_deref(),
        social_links_str.as_deref(),
        body.custom_css.as_deref(),
    )?;

    Ok(Json(user))
}

/// POST /api/users/@me/avatar — upload d'avatar
async fn upload_avatar(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    while let Some(mut field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if field.name() == Some("image") || field.name() == Some("file") {
            let data = read_image_field(&mut field).await?;
            let filename = format!("{}.png", auth.user_id);
            let path = PathBuf::from(&state.config.data_dir).join("avatars").join(&filename);
            tokio::fs::write(&path, &data).await?;

            let avatar_path = format!("avatars/{}", filename);
            let conn = state.pool.get()?;
            users::update_avatar(&conn, &auth.user_id, &avatar_path)?;

            return Ok(Json(serde_json::json!({ "avatar_path": avatar_path })));
        }
    }

    Err(AppError::BadRequest("Aucun fichier fourni".to_string()))
}

/// POST /api/users/@me/banner — upload de bannière
async fn upload_banner(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    while let Some(mut field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if field.name() == Some("image") || field.name() == Some("file") {
            let data = read_image_field(&mut field).await?;
            let filename = format!("{}_banner.png", auth.user_id);
            let path = PathBuf::from(&state.config.data_dir).join("banners").join(&filename);
            tokio::fs::write(&path, &data).await?;

            let banner_path = format!("banners/{}", filename);
            let conn = state.pool.get()?;
            users::update_banner(&conn, &auth.user_id, &banner_path)?;

            return Ok(Json(serde_json::json!({ "banner_path": banner_path })));
        }
    }

    Err(AppError::BadRequest("Aucun fichier fourni".to_string()))
}

/// GET /api/users/:id — récupère le profil public d'un utilisateur
async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<User>, AppError> {
    let conn = state.pool.get()?;
    let user = users::get_user_by_id(&conn, &user_id)?;
    Ok(Json(user))
}
