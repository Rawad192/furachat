// Routes stickers — CRUD stickers personnels

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use std::path::PathBuf;
use uuid::Uuid;

use crate::api::upload_helpers::read_image_field;
use crate::auth::middleware::AuthUser;
use crate::db::queries::stickers;
use crate::error::AppError;
use crate::AppState;

/// Routes stickers
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/stickers", get(get_stickers).post(create_sticker))
        .route("/api/stickers/{id}", delete(delete_sticker))
}

/// GET /api/stickers — liste les stickers de l'utilisateur
async fn get_stickers(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<stickers::Sticker>>, AppError> {
    let conn = state.pool.get()?;
    let sticker_list = stickers::get_user_stickers(&conn, &auth.user_id)?;
    Ok(Json(sticker_list))
}

/// POST /api/stickers — crée un sticker (multipart: name + file)
async fn create_sticker(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<stickers::Sticker>, AppError> {
    let mut name = String::new();
    let mut file_data = None;

    while let Some(mut field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        match field.name() {
            Some("name") => {
                name = field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
            }
            Some("file") => {
                file_data = Some(read_image_field(&mut field).await?);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return Err(AppError::BadRequest("Le nom du sticker est requis".to_string()));
    }
    let data = file_data.ok_or_else(|| AppError::BadRequest("Le fichier du sticker est requis".to_string()))?;

    let filename = format!("{}.png", Uuid::new_v4());
    let path = PathBuf::from(&state.config.data_dir).join("stickers").join(&filename);
    tokio::fs::write(&path, &data).await?;

    let file_path = format!("stickers/{}", filename);
    let conn = state.pool.get()?;
    let sticker = stickers::create_sticker(&conn, &auth.user_id, &name, &file_path)?;

    Ok(Json(sticker))
}

/// DELETE /api/stickers/:id — supprime un sticker
async fn delete_sticker(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(sticker_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;
    stickers::delete_sticker(&conn, &sticker_id, &auth.user_id)?;
    Ok(StatusCode::NO_CONTENT)
}
