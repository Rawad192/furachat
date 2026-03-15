// Requêtes SQL pour les utilisateurs

use rusqlite::Connection;

use crate::error::AppError;
use crate::models::user::{User, UserRow};

/// Récupère un utilisateur par son ID
pub fn get_user_by_id(conn: &Connection, user_id: &str) -> Result<User, AppError> {
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

/// Met à jour le profil d'un utilisateur
pub fn update_user_profile(
    conn: &Connection,
    user_id: &str,
    username: Option<&str>,
    bio: Option<&str>,
    status_text: Option<&str>,
    status_emoji: Option<&str>,
    social_links: Option<&str>,
    custom_css: Option<&str>,
) -> Result<User, AppError> {
    let mut updates = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = username {
        updates.push("username = ?");
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = bio {
        updates.push("bio = ?");
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = status_text {
        updates.push("status_text = ?");
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = status_emoji {
        updates.push("status_emoji = ?");
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = social_links {
        updates.push("social_links = ?");
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = custom_css {
        updates.push("custom_css = ?");
        params.push(Box::new(v.to_string()));
    }

    if updates.is_empty() {
        return get_user_by_id(conn, user_id);
    }

    updates.push("updated_at = CURRENT_TIMESTAMP");
    params.push(Box::new(user_id.to_string()));

    let sql = format!(
        "UPDATE users SET {} WHERE id = ?",
        updates.join(", ")
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, param_refs.as_slice())?;

    get_user_by_id(conn, user_id)
}

/// Met à jour le chemin de l'avatar
pub fn update_avatar(conn: &Connection, user_id: &str, path: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE users SET avatar_path = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        rusqlite::params![path, user_id],
    )?;
    Ok(())
}

/// Met à jour le chemin de la bannière
pub fn update_banner(conn: &Connection, user_id: &str, path: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE users SET banner_path = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        rusqlite::params![path, user_id],
    )?;
    Ok(())
}
