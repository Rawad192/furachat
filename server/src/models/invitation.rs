// Modèle invitation — invitations de serveur

use serde::{Deserialize, Serialize};

/// Invitation de serveur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: String,
    pub server_id: String,
    pub created_by: String,
    pub code: String,
    pub expires_at: Option<String>,
    pub max_uses: Option<i32>,
    pub use_count: i32,
    pub created_at: String,
}

/// Détails publics d'une invitation (pour prévisualisation avant de rejoindre)
#[derive(Debug, Serialize)]
pub struct InviteInfo {
    pub invite: Invitation,
    pub server_name: String,
    pub member_count: i64,
}

/// Corps de requête pour créer une invitation
#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub expires_in_seconds: Option<i64>,
    pub max_uses: Option<i32>,
}
