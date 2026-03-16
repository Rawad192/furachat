// Module API — assemblage de toutes les routes REST

pub mod auth_routes;
pub mod badge_routes;
pub mod upload_helpers;
pub mod channel_routes;
pub mod forum_routes;
pub mod friend_routes;
pub mod health_routes;
pub mod invitation_routes;
pub mod message_routes;
pub mod moderation_routes;
pub mod role_routes;
pub mod server_routes;
pub mod sticker_routes;
pub mod upload_routes;
pub mod user_routes;

use axum::Router;
use crate::AppState;

/// Construit le routeur API complet
pub fn create_router() -> Router<AppState> {
    Router::new()
        .merge(health_routes::routes())
        .merge(auth_routes::routes())
        .merge(user_routes::routes())
        .merge(server_routes::routes())
        .merge(channel_routes::routes())
        .merge(message_routes::routes())
        .merge(role_routes::routes())
        .merge(friend_routes::routes())
        .merge(invitation_routes::routes())
        .merge(forum_routes::routes())
        .merge(moderation_routes::routes())
        .merge(sticker_routes::routes())
        .merge(badge_routes::routes())
        .merge(upload_routes::routes())
}
