// Requêtes SQL pour les invitations de serveur

use chrono::Utc;
use rand::Rng;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::invitation::{InviteInfo, Invitation};

/// Génère un code d'invitation aléatoire de 8 caractères
fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Crée une nouvelle invitation
pub fn create_invitation(
    conn: &Connection,
    server_id: &str,
    created_by: &str,
    expires_in_seconds: Option<i64>,
    max_uses: Option<i32>,
) -> Result<Invitation, AppError> {
    let id = Uuid::new_v4().to_string();
    let code = generate_invite_code();

    let expires_at = expires_in_seconds.map(|secs| {
        (Utc::now() + chrono::Duration::seconds(secs)).format("%Y-%m-%d %H:%M:%S").to_string()
    });

    conn.execute(
        "INSERT INTO invitations (id, server_id, created_by, code, expires_at, max_uses) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![&id, server_id, created_by, &code, &expires_at, max_uses],
    )?;

    get_invitation(conn, &id)
}

/// Récupère une invitation par son ID
pub fn get_invitation(conn: &Connection, invite_id: &str) -> Result<Invitation, AppError> {
    conn.query_row(
        "SELECT id, server_id, created_by, code, expires_at, max_uses, use_count, created_at FROM invitations WHERE id = ?1",
        [invite_id],
        |row| {
            Ok(Invitation {
                id: row.get(0)?,
                server_id: row.get(1)?,
                created_by: row.get(2)?,
                code: row.get(3)?,
                expires_at: row.get(4)?,
                max_uses: row.get(5)?,
                use_count: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Invitation introuvable".to_string()))
}

/// Récupère les infos d'une invitation par son code
pub fn get_invite_by_code(conn: &Connection, code: &str) -> Result<InviteInfo, AppError> {
    let invite = conn.query_row(
        "SELECT id, server_id, created_by, code, expires_at, max_uses, use_count, created_at FROM invitations WHERE code = ?1",
        [code],
        |row| {
            Ok(Invitation {
                id: row.get(0)?,
                server_id: row.get(1)?,
                created_by: row.get(2)?,
                code: row.get(3)?,
                expires_at: row.get(4)?,
                max_uses: row.get(5)?,
                use_count: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Invitation introuvable".to_string()))?;

    // Vérifie l'expiration
    if let Some(ref expires) = invite.expires_at {
        if let Ok(exp_time) = chrono::NaiveDateTime::parse_from_str(expires, "%Y-%m-%d %H:%M:%S") {
            if exp_time < Utc::now().naive_utc() {
                return Err(AppError::BadRequest("Cette invitation a expiré".to_string()));
            }
        }
    }

    // Vérifie le nombre d'utilisations
    if let Some(max) = invite.max_uses {
        if invite.use_count >= max {
            return Err(AppError::BadRequest("Cette invitation a atteint son nombre maximal d'utilisations".to_string()));
        }
    }

    // Récupère le nom du serveur et le nombre de membres
    let server_name: String = conn.query_row(
        "SELECT name FROM servers WHERE id = ?1",
        [&invite.server_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("Serveur introuvable".to_string()))?;

    let member_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM server_members WHERE server_id = ?1",
        [&invite.server_id],
        |row| row.get(0),
    )?;

    Ok(InviteInfo {
        invite,
        server_name,
        member_count,
    })
}

/// Utilise une invitation pour rejoindre un serveur
pub fn use_invitation(conn: &Connection, code: &str, user_id: &str) -> Result<String, AppError> {
    let info = get_invite_by_code(conn, code)?;

    // Incrémente le compteur d'utilisations
    conn.execute(
        "UPDATE invitations SET use_count = use_count + 1 WHERE code = ?1",
        [code],
    )?;

    Ok(info.invite.server_id)
}
