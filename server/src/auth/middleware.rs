// Middleware d'authentification — extrait l'utilisateur courant depuis le JWT

use axum::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::auth::jwt::validate_token;
use crate::error::AppError;

/// Extracteur Axum qui vérifie le JWT et fournit les claims de l'utilisateur
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
}

/// Clé secrète JWT stockée dans les extensions de la requête
#[derive(Clone)]
pub struct JwtSecret(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Récupère la clé JWT depuis les extensions
        let jwt_secret = parts
            .extensions
            .get::<JwtSecret>()
            .ok_or_else(|| AppError::Internal("Configuration JWT manquante".to_string()))?;

        // Cherche le token dans le header Authorization: Bearer <token>
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("Token manquant".to_string()))?;

        let claims = validate_token(token, &jwt_secret.0)?;

        Ok(AuthUser {
            user_id: claims.sub,
            username: claims.username,
        })
    }
}
