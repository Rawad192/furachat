// Requêtes SQL pour la modération (kick, ban, mute, audit log)

use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;

/// Entrée du journal d'audit
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub server_id: String,
    pub actor_id: String,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub details: serde_json::Value,
    pub created_at: String,
}

/// Fonction utilitaire pour mapper une ligne d'audit
fn map_audit_row(row: &rusqlite::Row) -> Result<AuditLogEntry, rusqlite::Error> {
    let details_str: String = row.get(6)?;
    let details = serde_json::from_str(&details_str).unwrap_or(serde_json::json!({}));
    Ok(AuditLogEntry {
        id: row.get(0)?,
        server_id: row.get(1)?,
        actor_id: row.get(2)?,
        action: row.get(3)?,
        target_type: row.get(4)?,
        target_id: row.get(5)?,
        details,
        created_at: row.get(7)?,
    })
}

/// Crée une entrée dans le journal d'audit
pub fn create_audit_log(
    conn: &Connection,
    server_id: &str,
    actor_id: &str,
    action: &str,
    target_type: Option<&str>,
    target_id: Option<&str>,
    details: &serde_json::Value,
) -> Result<(), AppError> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO audit_logs (id, server_id, actor_id, action, target_type, target_id, details) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![&id, server_id, actor_id, action, target_type, target_id, details.to_string()],
    )?;
    Ok(())
}

/// Récupère le journal d'audit d'un serveur
pub fn get_audit_log(
    conn: &Connection,
    server_id: &str,
    limit: i64,
    before_id: Option<&str>,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let entries = if let Some(before) = before_id {
        let before_time: String = conn.query_row(
            "SELECT created_at FROM audit_logs WHERE id = ?1",
            [before],
            |row| row.get(0),
        ).map_err(|_| AppError::NotFound("Entrée d'audit introuvable".to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, server_id, actor_id, action, target_type, target_id, details, created_at FROM audit_logs WHERE server_id = ?1 AND created_at < ?2 ORDER BY created_at DESC LIMIT ?3"
        )?;

        let rows = stmt.query_map(params![server_id, &before_time, limit], map_audit_row)?;
        rows.filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, server_id, actor_id, action, target_type, target_id, details, created_at FROM audit_logs WHERE server_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![server_id, limit], map_audit_row)?;
        rows.filter_map(|r| r.ok()).collect()
    };

    Ok(entries)
}

/// Ban un utilisateur
pub fn ban_user(
    conn: &Connection,
    server_id: &str,
    user_id: &str,
    banned_by: &str,
    reason: &str,
    duration_seconds: Option<i64>,
) -> Result<(), AppError> {
    let expires_at = duration_seconds.map(|secs| {
        (Utc::now() + chrono::Duration::seconds(secs)).format("%Y-%m-%d %H:%M:%S").to_string()
    });

    conn.execute(
        "INSERT OR REPLACE INTO bans (server_id, user_id, banned_by, reason, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![server_id, user_id, banned_by, reason, &expires_at],
    )?;

    // Retire le membre du serveur
    conn.execute(
        "DELETE FROM server_members WHERE server_id = ?1 AND user_id = ?2",
        params![server_id, user_id],
    )?;

    // Log d'audit
    create_audit_log(
        conn, server_id, banned_by, "ban",
        Some("user"), Some(user_id),
        &serde_json::json!({"reason": reason}),
    )?;

    Ok(())
}

/// Déban un utilisateur
pub fn unban_user(conn: &Connection, server_id: &str, user_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM bans WHERE server_id = ?1 AND user_id = ?2",
        params![server_id, user_id],
    )?;
    Ok(())
}

/// Vérifie si un utilisateur est banni
pub fn is_banned(conn: &Connection, server_id: &str, user_id: &str) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bans WHERE server_id = ?1 AND user_id = ?2 AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)",
        params![server_id, user_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Mute un utilisateur
pub fn mute_user(
    conn: &Connection,
    server_id: &str,
    user_id: &str,
    muted_by: &str,
    reason: &str,
    duration_seconds: Option<i64>,
) -> Result<(), AppError> {
    let expires_at = duration_seconds.map(|secs| {
        (Utc::now() + chrono::Duration::seconds(secs)).format("%Y-%m-%d %H:%M:%S").to_string()
    });

    conn.execute(
        "INSERT OR REPLACE INTO mutes (server_id, user_id, muted_by, reason, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![server_id, user_id, muted_by, reason, &expires_at],
    )?;

    create_audit_log(
        conn, server_id, muted_by, "mute",
        Some("user"), Some(user_id),
        &serde_json::json!({"reason": reason}),
    )?;

    Ok(())
}

/// Unmute un utilisateur
pub fn unmute_user(conn: &Connection, server_id: &str, user_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM mutes WHERE server_id = ?1 AND user_id = ?2",
        params![server_id, user_id],
    )?;
    Ok(())
}

/// Kick un utilisateur
pub fn kick_user(
    conn: &Connection,
    server_id: &str,
    user_id: &str,
    kicked_by: &str,
    reason: &str,
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM server_members WHERE server_id = ?1 AND user_id = ?2",
        params![server_id, user_id],
    )?;

    create_audit_log(
        conn, server_id, kicked_by, "kick",
        Some("user"), Some(user_id),
        &serde_json::json!({"reason": reason}),
    )?;

    Ok(())
}
