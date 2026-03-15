// Routes d'authentification — inscription et connexion

use axum::{extract::State, routing::post, Json, Router};
use uuid::Uuid;

use crate::auth::{jwt, password};
use crate::error::AppError;
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest, User, UserRow};
use crate::AppState;

/// Routes d'authentification
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
}

/// POST /api/auth/register — crée un nouveau compte
async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validation du pseudo
    if body.username.trim().is_empty() || body.username.len() > 32 {
        return Err(AppError::BadRequest(
            "Le pseudo doit faire entre 1 et 32 caractères".to_string(),
        ));
    }

    // Validation du mot de passe
    if body.password.len() < 4 {
        return Err(AppError::BadRequest(
            "Le mot de passe doit faire au moins 4 caractères".to_string(),
        ));
    }

    let conn = state.pool.get()?;

    // Vérifie que le pseudo n'est pas déjà pris
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM users WHERE username = ?1",
            [&body.username],
            |row| row.get(0),
        )?;

    if exists {
        return Err(AppError::Conflict("Ce pseudo est déjà pris".to_string()));
    }

    // Hache le mot de passe
    let password_hash = password::hash_password(&body.password)?;

    // Crée l'utilisateur
    let user_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO users (id, username, password_hash) VALUES (?1, ?2, ?3)",
        rusqlite::params![&user_id, &body.username, &password_hash],
    )?;

    // Récupère l'utilisateur créé
    let user = get_user_by_id(&conn, &user_id)?;

    // Génère le token JWT
    let token = jwt::create_token(
        &user.id,
        &user.username,
        &state.config.jwt_secret,
        state.config.jwt_expiration_seconds,
    )?;

    Ok(Json(AuthResponse {
        token,
        user,
    }))
}

/// POST /api/auth/login — se connecte avec un compte existant
async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let conn = state.pool.get()?;

    // Cherche l'utilisateur par pseudo
    let row: Result<UserRow, _> = conn.query_row(
        "SELECT id, username, password_hash, avatar_path, banner_path, bio, status_text, status_emoji, custom_css, social_links, created_at, updated_at FROM users WHERE username = ?1",
        [&body.username],
        |row| {
            Ok(UserRow {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                avatar_path: row.get(3)?,
                banner_path: row.get(4)?,
                bio: row.get(5)?,
                status_text: row.get(6)?,
                status_emoji: row.get(7)?,
                custom_css: row.get(8)?,
                social_links: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        },
    );

    let user_row = row.map_err(|_| {
        AppError::Unauthorized("Pseudo ou mot de passe incorrect".to_string())
    })?;

    // Vérifie le mot de passe
    let valid = password::verify_password(&body.password, &user_row.password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized(
            "Pseudo ou mot de passe incorrect".to_string(),
        ));
    }

    let user: User = user_row.into();

    // Génère le token JWT
    let token = jwt::create_token(
        &user.id,
        &user.username,
        &state.config.jwt_secret,
        state.config.jwt_expiration_seconds,
    )?;

    Ok(Json(AuthResponse { token, user }))
}

/// Récupère un utilisateur par son ID (vue publique)
fn get_user_by_id(
    conn: &rusqlite::Connection,
    user_id: &str,
) -> Result<User, AppError> {
    let row = conn.query_row(
        "SELECT id, username, password_hash, avatar_path, banner_path, bio, status_text, status_emoji, custom_css, social_links, created_at, updated_at FROM users WHERE id = ?1",
        [user_id],
        |row| {
            Ok(UserRow {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                avatar_path: row.get(3)?,
                banner_path: row.get(4)?,
                bio: row.get(5)?,
                status_text: row.get(6)?,
                status_emoji: row.get(7)?,
                custom_css: row.get(8)?,
                social_links: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Utilisateur introuvable".to_string()))?;

    Ok(row.into())
}
