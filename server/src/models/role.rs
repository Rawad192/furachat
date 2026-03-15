// Modèle rôle — rôles de serveur et permissions

use serde::{Deserialize, Serialize};

/// Rôle d'un serveur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub color: String,
    pub position: i32,
    pub permissions: serde_json::Value,
    pub is_default: bool,
    pub created_at: String,
}

/// Corps de requête pour créer un rôle
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub color: Option<String>,
    pub permissions: Option<serde_json::Value>,
}

/// Corps de requête pour modifier un rôle
#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub permissions: Option<serde_json::Value>,
    pub position: Option<i32>,
}

/// Override de permissions pour un salon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPermissionOverride {
    pub channel_id: String,
    pub role_id: String,
    pub allow: serde_json::Value,
    pub deny: serde_json::Value,
}
