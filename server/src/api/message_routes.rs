// Routes messages — chargement des messages (l'envoi se fait principalement via WebSocket)

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::auth::middleware::AuthUser;
use crate::db::queries::{channels, messages, servers};
use crate::error::AppError;
use crate::models::message::Message;
use crate::AppState;

/// Routes messages
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/channels/{id}/messages", get(get_messages))
}

/// Paramètres de requête pour le chargement des messages
#[derive(Debug, Deserialize)]
struct MessagesQuery {
    before: Option<String>,
    limit: Option<i64>,
}

/// GET /api/channels/:id/messages — charge les messages d'un salon
async fn get_messages(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<Vec<Message>>, AppError> {
    let conn = state.pool.get()?;

    // Vérifie l'accès
    let server_id = channels::get_channel_server_id(&conn, &channel_id)?;
    if !servers::is_member(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Accès refusé".to_string()));
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let msgs = messages::get_channel_messages(&conn, &channel_id, query.before.as_deref(), limit)?;

    Ok(Json(msgs))
}
