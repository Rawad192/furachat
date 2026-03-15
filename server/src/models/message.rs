// Modèle message — messages de salon et messages privés

use serde::{Deserialize, Serialize};

/// Message dans un salon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub author_id: String,
    pub author_username: String,
    pub author_avatar: Option<String>,
    pub content: String,
    pub file_path: Option<String>,
    pub reply_to_id: Option<String>,
    pub edited: bool,
    pub reactions: Vec<Reaction>,
    pub created_at: String,
    pub updated_at: String,
}

/// Message privé (DM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub id: String,
    pub sender_id: String,
    pub sender_username: String,
    pub sender_avatar: Option<String>,
    pub receiver_id: String,
    pub content: String,
    pub file_path: Option<String>,
    pub edited: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Réaction sur un message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub users: Vec<String>,
}

/// Corps de requête pour envoyer un message (via REST, rarement utilisé)
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub reply_to_id: Option<String>,
}

/// Corps de requête pour modifier un message
#[derive(Debug, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}
