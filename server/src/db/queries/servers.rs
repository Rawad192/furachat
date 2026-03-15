// Requêtes SQL pour les serveurs

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::channel::{Category, Channel};
use crate::models::role::Role;
use crate::models::server::{Server, ServerDetails, ServerMember};

/// Crée un nouveau serveur avec un rôle @everyone et un salon #général par défaut
pub fn create_server(conn: &Connection, name: &str, owner_id: &str) -> Result<Server, AppError> {
    let server_id = Uuid::new_v4().to_string();
    let role_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4().to_string();
    let category_id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO servers (id, name, owner_id) VALUES (?1, ?2, ?3)",
        params![&server_id, name, owner_id],
    )?;

    // Ajoute le propriétaire comme membre
    conn.execute(
        "INSERT INTO server_members (server_id, user_id) VALUES (?1, ?2)",
        params![&server_id, owner_id],
    )?;

    // Crée le rôle @everyone par défaut
    let default_perms = serde_json::json!({
        "send_messages": true,
        "send_files": true,
        "connect_voice": true,
        "speak_voice": true,
        "use_video": true,
        "share_screen": true
    });
    conn.execute(
        "INSERT INTO roles (id, server_id, name, color, position, permissions, is_default) VALUES (?1, ?2, '@everyone', '#ffffff', 0, ?3, TRUE)",
        params![&role_id, &server_id, default_perms.to_string()],
    )?;

    // Crée une catégorie par défaut
    conn.execute(
        "INSERT INTO categories (id, server_id, name, position) VALUES (?1, ?2, 'Général', 0)",
        params![&category_id, &server_id],
    )?;

    // Crée le salon #général
    conn.execute(
        "INSERT INTO channels (id, server_id, category_id, name, type, position) VALUES (?1, ?2, ?3, 'général', 'text', 0)",
        params![&channel_id, &server_id, &category_id],
    )?;

    get_server(conn, &server_id)
}

/// Récupère un serveur par son ID
pub fn get_server(conn: &Connection, server_id: &str) -> Result<Server, AppError> {
    conn.query_row(
        "SELECT id, name, icon_path, banner_path, owner_id, created_at, updated_at FROM servers WHERE id = ?1",
        [server_id],
        |row| {
            Ok(Server {
                id: row.get(0)?,
                name: row.get(1)?,
                icon_path: row.get(2)?,
                banner_path: row.get(3)?,
                owner_id: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Serveur introuvable".to_string()))
}

/// Récupère les détails complets d'un serveur
pub fn get_server_details(conn: &Connection, server_id: &str) -> Result<ServerDetails, AppError> {
    let server = get_server(conn, server_id)?;
    let channels = get_server_channels(conn, server_id)?;
    let categories = get_server_categories(conn, server_id)?;
    let members = get_server_members(conn, server_id)?;
    let roles = get_server_roles(conn, server_id)?;

    Ok(ServerDetails {
        server,
        channels,
        categories,
        members,
        roles,
    })
}

/// Met à jour un serveur
pub fn update_server(conn: &Connection, server_id: &str, name: Option<&str>) -> Result<Server, AppError> {
    if let Some(name) = name {
        conn.execute(
            "UPDATE servers SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![name, server_id],
        )?;
    }
    get_server(conn, server_id)
}

/// Supprime un serveur
pub fn delete_server(conn: &Connection, server_id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM servers WHERE id = ?1", [server_id])?;
    Ok(())
}

/// Met à jour l'icône du serveur
pub fn update_server_icon(conn: &Connection, server_id: &str, path: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE servers SET icon_path = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![path, server_id],
    )?;
    Ok(())
}

/// Met à jour la bannière du serveur
pub fn update_server_banner(conn: &Connection, server_id: &str, path: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE servers SET banner_path = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![path, server_id],
    )?;
    Ok(())
}

/// Récupère la liste des serveurs d'un utilisateur
pub fn get_user_servers(conn: &Connection, user_id: &str) -> Result<Vec<Server>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.icon_path, s.banner_path, s.owner_id, s.created_at, s.updated_at FROM servers s JOIN server_members sm ON s.id = sm.server_id WHERE sm.user_id = ?1"
    )?;

    let servers = stmt.query_map([user_id], |row| {
        Ok(Server {
            id: row.get(0)?,
            name: row.get(1)?,
            icon_path: row.get(2)?,
            banner_path: row.get(3)?,
            owner_id: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(servers)
}

/// Vérifie si un utilisateur est membre d'un serveur
pub fn is_member(conn: &Connection, server_id: &str, user_id: &str) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM server_members WHERE server_id = ?1 AND user_id = ?2",
        params![server_id, user_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Ajoute un membre au serveur
pub fn add_member(conn: &Connection, server_id: &str, user_id: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR IGNORE INTO server_members (server_id, user_id) VALUES (?1, ?2)",
        params![server_id, user_id],
    )?;
    Ok(())
}

/// Retire un membre du serveur
pub fn remove_member(conn: &Connection, server_id: &str, user_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM server_members WHERE server_id = ?1 AND user_id = ?2",
        params![server_id, user_id],
    )?;
    Ok(())
}

/// Récupère les membres d'un serveur
pub fn get_server_members(conn: &Connection, server_id: &str) -> Result<Vec<ServerMember>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT sm.server_id, sm.user_id, u.username, sm.nickname, u.avatar_path, u.status_text, u.status_emoji, sm.joined_at FROM server_members sm JOIN users u ON sm.user_id = u.id WHERE sm.server_id = ?1"
    )?;

    let members: Vec<ServerMember> = stmt.query_map([server_id], |row| {
        Ok(ServerMember {
            server_id: row.get(0)?,
            user_id: row.get(1)?,
            username: row.get(2)?,
            nickname: row.get(3)?,
            avatar_path: row.get(4)?,
            status_text: row.get(5)?,
            status_emoji: row.get(6)?,
            joined_at: row.get(7)?,
            roles: Vec::new(), // Rempli ci-dessous
        })
    })?.filter_map(|r| r.ok()).collect();

    // Charge les rôles de chaque membre
    let mut result = Vec::new();
    for mut member in members {
        let mut role_stmt = conn.prepare(
            "SELECT role_id FROM member_roles WHERE server_id = ?1 AND user_id = ?2"
        )?;
        let roles: Vec<String> = role_stmt.query_map(
            params![server_id, &member.user_id],
            |row| row.get(0),
        )?.filter_map(|r| r.ok()).collect();
        member.roles = roles;
        result.push(member);
    }

    Ok(result)
}

/// Récupère les salons d'un serveur
pub fn get_server_channels(conn: &Connection, server_id: &str) -> Result<Vec<Channel>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, server_id, category_id, name, type, topic, position, is_archived, created_at, updated_at FROM channels WHERE server_id = ?1 ORDER BY position"
    )?;

    let channels = stmt.query_map([server_id], |row| {
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
    })?.filter_map(|r| r.ok()).collect();

    Ok(channels)
}

/// Récupère les catégories d'un serveur
pub fn get_server_categories(conn: &Connection, server_id: &str) -> Result<Vec<Category>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, server_id, name, position, created_at FROM categories WHERE server_id = ?1 ORDER BY position"
    )?;

    let categories = stmt.query_map([server_id], |row| {
        Ok(Category {
            id: row.get(0)?,
            server_id: row.get(1)?,
            name: row.get(2)?,
            position: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(categories)
}

/// Récupère les rôles d'un serveur
pub fn get_server_roles(conn: &Connection, server_id: &str) -> Result<Vec<Role>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, server_id, name, color, position, permissions, is_default, created_at FROM roles WHERE server_id = ?1 ORDER BY position DESC"
    )?;

    let roles = stmt.query_map([server_id], |row| {
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
    })?.filter_map(|r| r.ok()).collect();

    Ok(roles)
}

/// Vérifie si un utilisateur est le propriétaire du serveur
pub fn is_owner(conn: &Connection, server_id: &str, user_id: &str) -> Result<bool, AppError> {
    let owner_id: String = conn.query_row(
        "SELECT owner_id FROM servers WHERE id = ?1",
        [server_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("Serveur introuvable".to_string()))?;

    Ok(owner_id == user_id)
}
