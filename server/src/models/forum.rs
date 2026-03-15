// Modèle forum — posts et réponses

use serde::{Deserialize, Serialize};

/// Post de forum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPost {
    pub id: String,
    pub channel_id: String,
    pub author_id: String,
    pub author_username: String,
    pub author_avatar: Option<String>,
    pub title: String,
    pub content: String,
    pub reply_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Réponse à un post de forum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumReply {
    pub id: String,
    pub post_id: String,
    pub author_id: String,
    pub author_username: String,
    pub author_avatar: Option<String>,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Corps de requête pour créer un post de forum
#[derive(Debug, Deserialize)]
pub struct CreateForumPostRequest {
    pub title: String,
    pub content: String,
}

/// Corps de requête pour répondre à un post
#[derive(Debug, Deserialize)]
pub struct CreateForumReplyRequest {
    pub content: String,
}
