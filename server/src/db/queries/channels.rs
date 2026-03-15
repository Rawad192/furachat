// Requêtes SQL pour les salons et catégories

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::channel::{Category, Channel};

/// Crée un nouveau salon
pub fn create_channel(
    conn: &Connection,
    server_id: &str,
    name: &str,
    channel_type: &str,
    category_id: Option<&str>,
) -> Result<Channel, AppError> {
    let id = Uuid::new_v4().to_string();

    // Récupère la position maximale pour ce serveur
    let max_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) FROM channels WHERE server_id = ?1",
        [server_id],
        |row| row.get(0),
    ).unwrap_or(0);

    conn.execute(
        "INSERT INTO channels (id, server_id, category_id, name, type, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![&id, server_id, category_id, name, channel_type, max_pos + 1],
    )?;

    get_channel(conn, &id)
}

/// Récupère un salon par son ID
pub fn get_channel(conn: &Connection, channel_id: &str) -> Result<Channel, AppError> {
    conn.query_row(
        "SELECT id, server_id, category_id, name, type, topic, position, is_archived, created_at, updated_at FROM channels WHERE id = ?1",
        [channel_id],
        |row| {
            Ok(Channel {
                id: row.get(0)?,
                server_id: row.get(1)?,
                category_id: row.get(2)?,
                name: row.get(3)?,
                channel_type: row.get(4)?,
                topic: row.get(5)?,
                position: row.get(6)?,
                is_archived: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Salon introuvable".to_string()))
}

/// Met à jour un salon
pub fn update_channel(
    conn: &Connection,
    channel_id: &str,
    name: Option<&str>,
    topic: Option<&str>,
    is_archived: Option<bool>,
) -> Result<Channel, AppError> {
    if let Some(name) = name {
        conn.execute(
            "UPDATE channels SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![name, channel_id],
        )?;
    }
    if let Some(topic) = topic {
        conn.execute(
            "UPDATE channels SET topic = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![topic, channel_id],
        )?;
    }
    if let Some(archived) = is_archived {
        conn.execute(
            "UPDATE channels SET is_archived = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![archived, channel_id],
        )?;
    }

    get_channel(conn, channel_id)
}

/// Supprime un salon
pub fn delete_channel(conn: &Connection, channel_id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM channels WHERE id = ?1", [channel_id])?;
    Ok(())
}

/// Crée une catégorie
pub fn create_category(conn: &Connection, server_id: &str, name: &str) -> Result<Category, AppError> {
    let id = Uuid::new_v4().to_string();

    let max_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) FROM categories WHERE server_id = ?1",
        [server_id],
        |row| row.get(0),
    ).unwrap_or(0);

    conn.execute(
        "INSERT INTO categories (id, server_id, name, position) VALUES (?1, ?2, ?3, ?4)",
        params![&id, server_id, name, max_pos + 1],
    )?;

    get_category(conn, &id)
}

/// Récupère une catégorie
pub fn get_category(conn: &Connection, category_id: &str) -> Result<Category, AppError> {
    conn.query_row(
        "SELECT id, server_id, name, position, created_at FROM categories WHERE id = ?1",
        [category_id],
        |row| {
            Ok(Category {
                id: row.get(0)?,
                server_id: row.get(1)?,
                name: row.get(2)?,
                position: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Catégorie introuvable".to_string()))
}

/// Met à jour une catégorie
pub fn update_category(
    conn: &Connection,
    category_id: &str,
    name: Option<&str>,
    position: Option<i32>,
) -> Result<Category, AppError> {
    if let Some(name) = name {
        conn.execute(
            "UPDATE categories SET name = ?1 WHERE id = ?2",
            params![name, category_id],
        )?;
    }
    if let Some(pos) = position {
        conn.execute(
            "UPDATE categories SET position = ?1 WHERE id = ?2",
            params![pos, category_id],
        )?;
    }

    get_category(conn, category_id)
}

/// Supprime une catégorie
pub fn delete_category(conn: &Connection, category_id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM categories WHERE id = ?1", [category_id])?;
    Ok(())
}

/// Récupère le server_id d'un salon
pub fn get_channel_server_id(conn: &Connection, channel_id: &str) -> Result<String, AppError> {
    conn.query_row(
        "SELECT server_id FROM channels WHERE id = ?1",
        [channel_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("Salon introuvable".to_string()))
}
