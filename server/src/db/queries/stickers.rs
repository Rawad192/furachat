// Requêtes SQL pour les stickers et badges

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;

/// Sticker
#[derive(Debug, Clone, serde::Serialize)]
pub struct Sticker {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub file_path: String,
    pub created_at: String,
}

/// Badge
#[derive(Debug, Clone, serde::Serialize)]
pub struct Badge {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub icon_path: String,
    pub created_at: String,
}

/// Récupère les stickers d'un utilisateur
pub fn get_user_stickers(conn: &Connection, user_id: &str) -> Result<Vec<Sticker>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, owner_id, name, file_path, created_at FROM stickers WHERE owner_id = ?1"
    )?;

    let stickers = stmt.query_map([user_id], |row| {
        Ok(Sticker {
            id: row.get(0)?,
            owner_id: row.get(1)?,
            name: row.get(2)?,
            file_path: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(stickers)
}

/// Crée un sticker
pub fn create_sticker(conn: &Connection, owner_id: &str, name: &str, file_path: &str) -> Result<Sticker, AppError> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO stickers (id, owner_id, name, file_path) VALUES (?1, ?2, ?3, ?4)",
        params![&id, owner_id, name, file_path],
    )?;

    conn.query_row(
        "SELECT id, owner_id, name, file_path, created_at FROM stickers WHERE id = ?1",
        [&id],
        |row| {
            Ok(Sticker {
                id: row.get(0)?,
                owner_id: row.get(1)?,
                name: row.get(2)?,
                file_path: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    ).map_err(|_| AppError::Internal("Erreur lors de la création du sticker".to_string()))
}

/// Supprime un sticker
pub fn delete_sticker(conn: &Connection, sticker_id: &str, owner_id: &str) -> Result<(), AppError> {
    let affected = conn.execute(
        "DELETE FROM stickers WHERE id = ?1 AND owner_id = ?2",
        params![sticker_id, owner_id],
    )?;

    if affected == 0 {
        return Err(AppError::NotFound("Sticker introuvable ou non autorisé".to_string()));
    }

    Ok(())
}

/// Crée un badge de serveur
pub fn create_badge(conn: &Connection, server_id: &str, name: &str, icon_path: &str) -> Result<Badge, AppError> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO badges (id, server_id, name, icon_path) VALUES (?1, ?2, ?3, ?4)",
        params![&id, server_id, name, icon_path],
    )?;

    conn.query_row(
        "SELECT id, server_id, name, icon_path, created_at FROM badges WHERE id = ?1",
        [&id],
        |row| {
            Ok(Badge {
                id: row.get(0)?,
                server_id: row.get(1)?,
                name: row.get(2)?,
                icon_path: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    ).map_err(|_| AppError::Internal("Erreur lors de la création du badge".to_string()))
}

/// Attribue un badge à un utilisateur
pub fn award_badge(conn: &Connection, badge_id: &str, user_id: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR IGNORE INTO user_badges (user_id, badge_id) VALUES (?1, ?2)",
        params![user_id, badge_id],
    )?;
    Ok(())
}

/// Retire un badge d'un utilisateur
pub fn revoke_badge(conn: &Connection, badge_id: &str, user_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM user_badges WHERE user_id = ?1 AND badge_id = ?2",
        params![user_id, badge_id],
    )?;
    Ok(())
}

/// Récupère le server_id d'un badge
pub fn get_badge_server_id(conn: &Connection, badge_id: &str) -> Result<String, AppError> {
    conn.query_row(
        "SELECT server_id FROM badges WHERE id = ?1",
        [badge_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("Badge introuvable".to_string()))
}
