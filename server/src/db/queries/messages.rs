// Requêtes SQL pour les messages

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::message::{DirectMessage, Message, Reaction};

/// Fonction utilitaire pour mapper une ligne SQL en Message
fn map_message_row(row: &rusqlite::Row) -> Result<Message, rusqlite::Error> {
    Ok(Message {
        id: row.get(0)?,
        channel_id: row.get(1)?,
        author_id: row.get(2)?,
        author_username: row.get(3)?,
        author_avatar: row.get(4)?,
        content: row.get(5)?,
        file_path: row.get(6)?,
        reply_to_id: row.get(7)?,
        edited: row.get(8)?,
        reactions: Vec::new(),
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

/// Crée un nouveau message dans un salon
pub fn create_message(
    conn: &Connection,
    channel_id: &str,
    author_id: &str,
    content: &str,
    file_path: Option<&str>,
    reply_to_id: Option<&str>,
) -> Result<Message, AppError> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO messages (id, channel_id, author_id, content, file_path, reply_to_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![&id, channel_id, author_id, content, file_path, reply_to_id],
    )?;

    get_message(conn, &id)
}

/// Récupère un message par son ID
pub fn get_message(conn: &Connection, message_id: &str) -> Result<Message, AppError> {
    let msg = conn.query_row(
        "SELECT m.id, m.channel_id, m.author_id, u.username, u.avatar_path, m.content, m.file_path, m.reply_to_id, m.edited, m.created_at, m.updated_at FROM messages m JOIN users u ON m.author_id = u.id WHERE m.id = ?1",
        [message_id],
        |row| {
            Ok(Message {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                author_id: row.get(2)?,
                author_username: row.get(3)?,
                author_avatar: row.get(4)?,
                content: row.get(5)?,
                file_path: row.get(6)?,
                reply_to_id: row.get(7)?,
                edited: row.get(8)?,
                reactions: Vec::new(),
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Message introuvable".to_string()))?;

    let reactions = get_message_reactions(conn, &msg.id)?;
    Ok(Message { reactions, ..msg })
}

/// Récupère les messages d'un salon (paginé)
pub fn get_channel_messages(
    conn: &Connection,
    channel_id: &str,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<Message>, AppError> {
    let messages: Vec<Message> = if let Some(before) = before_id {
        let before_time: String = conn.query_row(
            "SELECT created_at FROM messages WHERE id = ?1",
            [before],
            |row| row.get(0),
        ).map_err(|_| AppError::NotFound("Message de référence introuvable".to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT m.id, m.channel_id, m.author_id, u.username, u.avatar_path, m.content, m.file_path, m.reply_to_id, m.edited, m.created_at, m.updated_at FROM messages m JOIN users u ON m.author_id = u.id WHERE m.channel_id = ?1 AND m.created_at < ?2 ORDER BY m.created_at DESC LIMIT ?3"
        )?;

        let rows = stmt.query_map(params![channel_id, &before_time, limit], map_message_row)?;
        rows.filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT m.id, m.channel_id, m.author_id, u.username, u.avatar_path, m.content, m.file_path, m.reply_to_id, m.edited, m.created_at, m.updated_at FROM messages m JOIN users u ON m.author_id = u.id WHERE m.channel_id = ?1 ORDER BY m.created_at DESC LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![channel_id, limit], map_message_row)?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Charge les réactions pour chaque message
    let mut result = Vec::new();
    for msg in messages {
        let reactions = get_message_reactions(conn, &msg.id)?;
        result.push(Message { reactions, ..msg });
    }

    // Inverse pour avoir l'ordre chronologique
    result.reverse();
    Ok(result)
}

/// Modifie un message
pub fn edit_message(conn: &Connection, message_id: &str, content: &str) -> Result<Message, AppError> {
    conn.execute(
        "UPDATE messages SET content = ?1, edited = TRUE, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![content, message_id],
    )?;
    get_message(conn, message_id)
}

/// Supprime un message
pub fn delete_message(conn: &Connection, message_id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM messages WHERE id = ?1", [message_id])?;
    Ok(())
}

/// Récupère les réactions d'un message, groupées par emoji
pub fn get_message_reactions(conn: &Connection, message_id: &str) -> Result<Vec<Reaction>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT emoji, user_id FROM reactions WHERE message_id = ?1 ORDER BY emoji, created_at"
    )?;

    let rows: Vec<(String, String)> = stmt.query_map([message_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?.filter_map(|r| r.ok()).collect();

    // Groupe par emoji
    let mut reactions: Vec<Reaction> = Vec::new();
    for (emoji, user_id) in rows {
        if let Some(reaction) = reactions.iter_mut().find(|r| r.emoji == emoji) {
            reaction.users.push(user_id);
        } else {
            reactions.push(Reaction {
                emoji,
                users: vec![user_id],
            });
        }
    }

    Ok(reactions)
}

/// Ajoute une réaction
pub fn add_reaction(conn: &Connection, message_id: &str, user_id: &str, emoji: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR IGNORE INTO reactions (message_id, user_id, emoji) VALUES (?1, ?2, ?3)",
        params![message_id, user_id, emoji],
    )?;
    Ok(())
}

/// Supprime une réaction
pub fn remove_reaction(conn: &Connection, message_id: &str, user_id: &str, emoji: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM reactions WHERE message_id = ?1 AND user_id = ?2 AND emoji = ?3",
        params![message_id, user_id, emoji],
    )?;
    Ok(())
}

/// Crée un message privé
pub fn create_dm(
    conn: &Connection,
    sender_id: &str,
    receiver_id: &str,
    content: &str,
    file_path: Option<&str>,
) -> Result<DirectMessage, AppError> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO direct_messages (id, sender_id, receiver_id, content, file_path) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&id, sender_id, receiver_id, content, file_path],
    )?;

    get_dm(conn, &id)
}

/// Récupère un DM par ID
pub fn get_dm(conn: &Connection, dm_id: &str) -> Result<DirectMessage, AppError> {
    conn.query_row(
        "SELECT d.id, d.sender_id, u.username, u.avatar_path, d.receiver_id, d.content, d.file_path, d.edited, d.created_at, d.updated_at FROM direct_messages d JOIN users u ON d.sender_id = u.id WHERE d.id = ?1",
        [dm_id],
        |row| {
            Ok(DirectMessage {
                id: row.get(0)?,
                sender_id: row.get(1)?,
                sender_username: row.get(2)?,
                sender_avatar: row.get(3)?,
                receiver_id: row.get(4)?,
                content: row.get(5)?,
                file_path: row.get(6)?,
                edited: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Message introuvable".to_string()))
}

/// Récupère l'auteur d'un message
pub fn get_message_author_id(conn: &Connection, message_id: &str) -> Result<String, AppError> {
    conn.query_row(
        "SELECT author_id FROM messages WHERE id = ?1",
        [message_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("Message introuvable".to_string()))
}

/// Récupère le channel_id d'un message
pub fn get_message_channel_id(conn: &Connection, message_id: &str) -> Result<String, AppError> {
    conn.query_row(
        "SELECT channel_id FROM messages WHERE id = ?1",
        [message_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("Message introuvable".to_string()))
}
