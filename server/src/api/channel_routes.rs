// Routes salon — CRUD salons et catégories

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, patch, post},
    Json, Router,
};

use crate::auth::middleware::AuthUser;
use crate::db::queries::{channels, servers};
use crate::error::AppError;
use crate::models::channel::{
    Category, Channel, CreateCategoryRequest, CreateChannelRequest,
    UpdateCategoryRequest, UpdateChannelRequest,
};
use crate::AppState;

/// Routes salon
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/servers/{id}/channels", post(create_channel))
        .route("/api/channels/{id}", patch(update_channel).delete(delete_channel))
        .route("/api/servers/{id}/categories", post(create_category))
        .route("/api/categories/{id}", patch(update_category).delete(delete_category))
}

/// POST /api/servers/:id/channels — crée un salon
async fn create_channel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
    Json(body): Json<CreateChannelRequest>,
) -> Result<Json<Channel>, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_member(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Vous n'êtes pas membre de ce serveur".to_string()));
    }

    // Vérifie le type de salon
    let valid_types = ["text", "voice", "video", "screen", "forum", "announcement", "nsfw"];
    if !valid_types.contains(&body.channel_type.as_str()) {
        return Err(AppError::BadRequest(format!("Type de salon invalide : {}", body.channel_type)));
    }

    let channel = channels::create_channel(
        &conn,
        &server_id,
        &body.name,
        &body.channel_type,
        body.category_id.as_deref(),
    )?;

    Ok(Json(channel))
}

/// PATCH /api/channels/:id — modifie un salon
async fn update_channel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
    Json(body): Json<UpdateChannelRequest>,
) -> Result<Json<Channel>, AppError> {
    let conn = state.pool.get()?;

    let server_id = channels::get_channel_server_id(&conn, &channel_id)?;
    if !servers::is_member(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Vous n'êtes pas membre de ce serveur".to_string()));
    }

    let channel = channels::update_channel(
        &conn,
        &channel_id,
        body.name.as_deref(),
        body.topic.as_deref(),
        body.is_archived,
    )?;

    Ok(Json(channel))
}

/// DELETE /api/channels/:id — supprime un salon
async fn delete_channel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    let server_id = channels::get_channel_server_id(&conn, &channel_id)?;
    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut supprimer des salons".to_string()));
    }

    channels::delete_channel(&conn, &channel_id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/servers/:id/categories — crée une catégorie
async fn create_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
    Json(body): Json<CreateCategoryRequest>,
) -> Result<Json<Category>, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut créer des catégories".to_string()));
    }

    let category = channels::create_category(&conn, &server_id, &body.name)?;
    Ok(Json(category))
}

/// PATCH /api/categories/:id — modifie une catégorie
async fn update_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(category_id): Path<String>,
    Json(body): Json<UpdateCategoryRequest>,
) -> Result<Json<Category>, AppError> {
    let conn = state.pool.get()?;

    let cat = channels::get_category(&conn, &category_id)?;
    if !servers::is_owner(&conn, &cat.server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut modifier les catégories".to_string()));
    }

    let category = channels::update_category(&conn, &category_id, body.name.as_deref(), body.position)?;
    Ok(Json(category))
}

/// DELETE /api/categories/:id — supprime une catégorie
async fn delete_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(category_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    let cat = channels::get_category(&conn, &category_id)?;
    if !servers::is_owner(&conn, &cat.server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut supprimer des catégories".to_string()));
    }

    channels::delete_category(&conn, &category_id)?;
    Ok(StatusCode::NO_CONTENT)
}
