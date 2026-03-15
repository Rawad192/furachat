// Gestion centralisée des erreurs du serveur FuraChat
// Convertit toutes les erreurs en réponses HTTP appropriées

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Erreur applicative centralisée
#[derive(Debug)]
pub enum AppError {
    /// Erreur d'authentification (401)
    Unauthorized(String),
    /// Accès refusé (403)
    Forbidden(String),
    /// Ressource introuvable (404)
    NotFound(String),
    /// Requête invalide (400)
    BadRequest(String),
    /// Conflit (409) — ex: pseudo déjà pris
    Conflict(String),
    /// Erreur interne du serveur (500)
    Internal(String),
    /// Erreur de base de données
    Database(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized(msg) => write!(f, "Non autorisé : {}", msg),
            Self::Forbidden(msg) => write!(f, "Accès refusé : {}", msg),
            Self::NotFound(msg) => write!(f, "Introuvable : {}", msg),
            Self::BadRequest(msg) => write!(f, "Requête invalide : {}", msg),
            Self::Conflict(msg) => write!(f, "Conflit : {}", msg),
            Self::Internal(msg) => write!(f, "Erreur interne : {}", msg),
            Self::Database(msg) => write!(f, "Erreur base de données : {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Self::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        tracing::error!("Erreur SQLite : {:?}", err);
        Self::Database(format!("Erreur base de données : {}", err))
    }
}

impl From<r2d2::Error> for AppError {
    fn from(err: r2d2::Error) -> Self {
        tracing::error!("Erreur pool de connexions : {:?}", err);
        Self::Database(format!("Erreur pool de connexions : {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::BadRequest(format!("Erreur de sérialisation JSON : {}", err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        tracing::error!("Erreur I/O : {:?}", err);
        Self::Internal(format!("Erreur I/O : {}", err))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::Unauthorized(format!("Erreur JWT : {}", err))
    }
}
