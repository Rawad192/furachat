// Routes amis — ajout, suppression, liste

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};

use crate::auth::middleware::AuthUser;
use crate::db::queries::friends;
use crate::error::AppError;
use crate::models::user::User;
use crate::AppState;

/// Routes amis
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/friends", get(get_friends))
        .route("/api/friends/{user_id}", post(add_friend).delete(remove_friend))
}

/// GET /api/friends — liste les amis
async fn get_friends(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<User>>, AppError> {
    let conn = state.pool.get()?;
    let friend_list = friends::get_friends(&conn, &auth.user_id)?;
    Ok(Json(friend_list))
}

/// POST /api/friends/:user_id — ajoute un ami
async fn add_friend(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(friend_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.pool.get()?;
    friends::add_friend(&conn, &auth.user_id, &friend_id)?;

    let friend = crate::db::queries::users::get_user_by_id(&conn, &friend_id)?;
    Ok(Json(serde_json::json!({ "friend": friend })))
}

/// DELETE /api/friends/:user_id — supprime un ami
async fn remove_friend(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(friend_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;
    friends::remove_friend(&conn, &auth.user_id, &friend_id)?;
    Ok(StatusCode::NO_CONTENT)
}
