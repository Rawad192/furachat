// Gestion des tokens JWT — création et validation

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Claims contenus dans le JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// ID de l'utilisateur
    pub sub: String,
    /// Pseudo de l'utilisateur
    pub username: String,
    /// Date d'expiration (timestamp Unix)
    pub exp: usize,
    /// Date d'émission (timestamp Unix)
    pub iat: usize,
}

/// Crée un nouveau token JWT pour un utilisateur
pub fn create_token(
    user_id: &str,
    username: &str,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let exp = (now + chrono::Duration::seconds(expiration_seconds)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Erreur de création du token : {}", e)))
}

/// Valide un token JWT et retourne les claims
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
