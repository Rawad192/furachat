// Modèle utilisateur — représentation en base et pour l'API

use serde::{Deserialize, Serialize};

/// Utilisateur complet (usage interne, inclut le hash du mot de passe)
#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub avatar_path: Option<String>,
    pub banner_path: Option<String>,
    pub bio: String,
    pub status_text: String,
    pub status_emoji: String,
    pub custom_css: String,
    pub social_links: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Utilisateur public (sans mot de passe, envoyé via l'API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub avatar_path: Option<String>,
    pub banner_path: Option<String>,
    pub bio: String,
    pub status_text: String,
    pub status_emoji: String,
    pub custom_css: String,
    pub social_links: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        let social_links = serde_json::from_str(&row.social_links)
            .unwrap_or_else(|_| serde_json::json!({}));
        Self {
            id: row.id,
            username: row.username,
            avatar_path: row.avatar_path,
            banner_path: row.banner_path,
            bio: row.bio,
            status_text: row.status_text,
            status_emoji: row.status_emoji,
            custom_css: row.custom_css,
            social_links,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Corps de requête pour l'inscription
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

/// Corps de requête pour la connexion
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Réponse d'authentification (inscription ou connexion)
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

/// Corps de requête pour la mise à jour du profil
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub bio: Option<String>,
    pub status_text: Option<String>,
    pub status_emoji: Option<String>,
    pub social_links: Option<serde_json::Value>,
    pub custom_css: Option<String>,
}
