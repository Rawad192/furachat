// Modèle serveur — représentation des serveurs et de leurs membres

use serde::{Deserialize, Serialize};

/// Serveur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub banner_path: Option<String>,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Membre d'un serveur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMember {
    pub server_id: String,
    pub user_id: String,
    pub username: String,
    pub nickname: Option<String>,
    pub avatar_path: Option<String>,
    pub status_text: String,
    pub status_emoji: String,
    pub joined_at: String,
    pub roles: Vec<String>,
}

/// Corps de requête pour créer un serveur
#[derive(Debug, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
}

/// Corps de requête pour modifier un serveur
#[derive(Debug, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
}

/// Détails complets d'un serveur (avec salons, membres, rôles)
#[derive(Debug, Serialize)]
pub struct ServerDetails {
    pub server: Server,
    pub channels: Vec<super::channel::Channel>,
    pub categories: Vec<super::channel::Category>,
    pub members: Vec<ServerMember>,
    pub roles: Vec<super::role::Role>,
}
