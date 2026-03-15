// Requêtes SQL pour les amis

use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::user::User;
use crate::db::queries::users::get_user_by_id;

/// Récupère la liste des amis d'un utilisateur
pub fn get_friends(conn: &Connection, user_id: &str) -> Result<Vec<User>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT friend_id FROM friends WHERE user_id = ?1"
    )?;

    let friend_ids: Vec<String> = stmt.query_map([user_id], |row| {
        row.get(0)
    })?.filter_map(|r| r.ok()).collect();

    let mut friends = Vec::new();
    for fid in friend_ids {
        if let Ok(user) = get_user_by_id(conn, &fid) {
            friends.push(user);
        }
    }

    Ok(friends)
}

/// Ajoute un ami (relation bidirectionnelle)
pub fn add_friend(conn: &Connection, user_id: &str, friend_id: &str) -> Result<(), AppError> {
    if user_id == friend_id {
        return Err(AppError::BadRequest("Impossible de s'ajouter soi-même en ami".to_string()));
    }

    conn.execute(
        "INSERT OR IGNORE INTO friends (user_id, friend_id) VALUES (?1, ?2)",
        params![user_id, friend_id],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO friends (user_id, friend_id) VALUES (?1, ?2)",
        params![friend_id, user_id],
    )?;

    Ok(())
}

/// Supprime un ami (relation bidirectionnelle)
pub fn remove_friend(conn: &Connection, user_id: &str, friend_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM friends WHERE user_id = ?1 AND friend_id = ?2",
        params![user_id, friend_id],
    )?;
    conn.execute(
        "DELETE FROM friends WHERE user_id = ?1 AND friend_id = ?2",
        params![friend_id, user_id],
    )?;

    Ok(())
}
