// Requêtes SQL pour les rôles

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::role::Role;

/// Crée un nouveau rôle
pub fn create_role(
    conn: &Connection,
    server_id: &str,
    name: &str,
    color: &str,
    permissions: &serde_json::Value,
) -> Result<Role, AppError> {
    let id = Uuid::new_v4().to_string();

    let max_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), 0) FROM roles WHERE server_id = ?1",
        [server_id],
        |row| row.get(0),
    ).unwrap_or(0);

    conn.execute(
        "INSERT INTO roles (id, server_id, name, color, position, permissions) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![&id, server_id, name, color, max_pos + 1, permissions.to_string()],
    )?;

    get_role(conn, &id)
}

/// Récupère un rôle par son ID
pub fn get_role(conn: &Connection, role_id: &str) -> Result<Role, AppError> {
    conn.query_row(
        "SELECT id, server_id, name, color, position, permissions, is_default, created_at FROM roles WHERE id = ?1",
        [role_id],
        |row| {
            let perms_str: String = row.get(5)?;
            let permissions = serde_json::from_str(&perms_str).unwrap_or(serde_json::json!({}));
            Ok(Role {
                id: row.get(0)?,
                server_id: row.get(1)?,
                name: row.get(2)?,
                color: row.get(3)?,
                position: row.get(4)?,
                permissions,
                is_default: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Rôle introuvable".to_string()))
}

/// Met à jour un rôle
pub fn update_role(
    conn: &Connection,
    role_id: &str,
    name: Option<&str>,
    color: Option<&str>,
    permissions: Option<&serde_json::Value>,
    position: Option<i32>,
) -> Result<Role, AppError> {
    if let Some(name) = name {
        conn.execute("UPDATE roles SET name = ?1 WHERE id = ?2", params![name, role_id])?;
    }
    if let Some(color) = color {
        conn.execute("UPDATE roles SET color = ?1 WHERE id = ?2", params![color, role_id])?;
    }
    if let Some(perms) = permissions {
        conn.execute("UPDATE roles SET permissions = ?1 WHERE id = ?2", params![perms.to_string(), role_id])?;
    }
    if let Some(pos) = position {
        conn.execute("UPDATE roles SET position = ?1 WHERE id = ?2", params![pos, role_id])?;
    }

    get_role(conn, role_id)
}

/// Supprime un rôle
pub fn delete_role(conn: &Connection, role_id: &str) -> Result<(), AppError> {
    // Interdit la suppression du rôle @everyone
    let is_default: bool = conn.query_row(
        "SELECT is_default FROM roles WHERE id = ?1",
        [role_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("Rôle introuvable".to_string()))?;

    if is_default {
        return Err(AppError::BadRequest("Impossible de supprimer le rôle @everyone".to_string()));
    }

    conn.execute("DELETE FROM roles WHERE id = ?1", [role_id])?;
    Ok(())
}

/// Assigne un rôle à un membre
pub fn assign_role(conn: &Connection, server_id: &str, user_id: &str, role_id: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR IGNORE INTO member_roles (server_id, user_id, role_id) VALUES (?1, ?2, ?3)",
        params![server_id, user_id, role_id],
    )?;
    Ok(())
}

/// Retire un rôle d'un membre
pub fn remove_role(conn: &Connection, server_id: &str, user_id: &str, role_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM member_roles WHERE server_id = ?1 AND user_id = ?2 AND role_id = ?3",
        params![server_id, user_id, role_id],
    )?;
    Ok(())
}

/// Récupère le server_id d'un rôle
pub fn get_role_server_id(conn: &Connection, role_id: &str) -> Result<String, AppError> {
    conn.query_row(
        "SELECT server_id FROM roles WHERE id = ?1",
        [role_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("Rôle introuvable".to_string()))
}
