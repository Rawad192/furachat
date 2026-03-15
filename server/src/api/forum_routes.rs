// Routes forum — posts et réponses

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::auth::middleware::AuthUser;
use crate::db::queries::{forum, servers};
use crate::error::AppError;
use crate::models::forum::{CreateForumPostRequest, CreateForumReplyRequest, ForumPost, ForumReply};
use crate::AppState;

/// Routes forum (intégrées via les salons)
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/channels/{id}/posts", get(get_posts).post(create_post))
        .route("/api/posts/{id}/replies", get(get_replies).post(create_reply))
}

/// GET /api/channels/:id/posts — liste les posts d'un salon forum
async fn get_posts(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
) -> Result<Json<Vec<ForumPost>>, AppError> {
    let conn = state.pool.get()?;

    // Vérifie l'accès au serveur via le salon
    let server_id = crate::db::queries::channels::get_channel_server_id(&conn, &channel_id)?;
    if !servers::is_member(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Accès refusé".to_string()));
    }

    let posts = forum::get_channel_posts(&conn, &channel_id)?;
    Ok(Json(posts))
}

/// POST /api/channels/:id/posts — crée un post
async fn create_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
    Json(body): Json<CreateForumPostRequest>,
) -> Result<Json<ForumPost>, AppError> {
    let conn = state.pool.get()?;

    let server_id = crate::db::queries::channels::get_channel_server_id(&conn, &channel_id)?;
    if !servers::is_member(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Accès refusé".to_string()));
    }

    let post = forum::create_post(&conn, &channel_id, &auth.user_id, &body.title, &body.content)?;
    Ok(Json(post))
}

/// GET /api/posts/:id/replies — liste les réponses d'un post
async fn get_replies(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(post_id): Path<String>,
) -> Result<Json<Vec<ForumReply>>, AppError> {
    let conn = state.pool.get()?;
    let replies = forum::get_post_replies(&conn, &post_id)?;
    Ok(Json(replies))
}

/// POST /api/posts/:id/replies — crée une réponse
async fn create_reply(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(post_id): Path<String>,
    Json(body): Json<CreateForumReplyRequest>,
) -> Result<Json<ForumReply>, AppError> {
    let conn = state.pool.get()?;
    let reply = forum::create_reply(&conn, &post_id, &auth.user_id, &body.content)?;
    Ok(Json(reply))
}
