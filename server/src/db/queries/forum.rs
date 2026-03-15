// Requêtes SQL pour le forum

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::forum::{ForumPost, ForumReply};

/// Crée un post de forum
pub fn create_post(
    conn: &Connection,
    channel_id: &str,
    author_id: &str,
    title: &str,
    content: &str,
) -> Result<ForumPost, AppError> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO forum_posts (id, channel_id, author_id, title, content) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&id, channel_id, author_id, title, content],
    )?;

    get_post(conn, &id)
}

/// Récupère un post par son ID
pub fn get_post(conn: &Connection, post_id: &str) -> Result<ForumPost, AppError> {
    let reply_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM forum_replies WHERE post_id = ?1",
        [post_id],
        |row| row.get(0),
    ).unwrap_or(0);

    conn.query_row(
        "SELECT p.id, p.channel_id, p.author_id, u.username, u.avatar_path, p.title, p.content, p.created_at, p.updated_at FROM forum_posts p JOIN users u ON p.author_id = u.id WHERE p.id = ?1",
        [post_id],
        |row| {
            Ok(ForumPost {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                author_id: row.get(2)?,
                author_username: row.get(3)?,
                author_avatar: row.get(4)?,
                title: row.get(5)?,
                content: row.get(6)?,
                reply_count,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Post introuvable".to_string()))
}

/// Récupère les posts d'un salon forum
pub fn get_channel_posts(conn: &Connection, channel_id: &str) -> Result<Vec<ForumPost>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.channel_id, p.author_id, u.username, u.avatar_path, p.title, p.content, p.created_at, p.updated_at FROM forum_posts p JOIN users u ON p.author_id = u.id WHERE p.channel_id = ?1 ORDER BY p.created_at DESC"
    )?;

    let posts: Vec<ForumPost> = stmt.query_map([channel_id], |row| {
        let post_id: String = row.get(0)?;
        Ok((post_id, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?))
    })?.filter_map(|r| r.ok()).map(|(id, channel_id, author_id, username, avatar, title, content, created, updated): (String, String, String, String, Option<String>, String, String, String, String)| {
        let reply_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM forum_replies WHERE post_id = ?1",
            [&id],
            |row| row.get(0),
        ).unwrap_or(0);

        ForumPost {
            id,
            channel_id,
            author_id,
            author_username: username,
            author_avatar: avatar,
            title,
            content,
            reply_count,
            created_at: created,
            updated_at: updated,
        }
    }).collect();

    Ok(posts)
}

/// Crée une réponse à un post
pub fn create_reply(
    conn: &Connection,
    post_id: &str,
    author_id: &str,
    content: &str,
) -> Result<ForumReply, AppError> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO forum_replies (id, post_id, author_id, content) VALUES (?1, ?2, ?3, ?4)",
        params![&id, post_id, author_id, content],
    )?;

    get_reply(conn, &id)
}

/// Récupère une réponse par son ID
pub fn get_reply(conn: &Connection, reply_id: &str) -> Result<ForumReply, AppError> {
    conn.query_row(
        "SELECT r.id, r.post_id, r.author_id, u.username, u.avatar_path, r.content, r.created_at, r.updated_at FROM forum_replies r JOIN users u ON r.author_id = u.id WHERE r.id = ?1",
        [reply_id],
        |row| {
            Ok(ForumReply {
                id: row.get(0)?,
                post_id: row.get(1)?,
                author_id: row.get(2)?,
                author_username: row.get(3)?,
                author_avatar: row.get(4)?,
                content: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    ).map_err(|_| AppError::NotFound("Réponse introuvable".to_string()))
}

/// Récupère les réponses d'un post
pub fn get_post_replies(conn: &Connection, post_id: &str) -> Result<Vec<ForumReply>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.post_id, r.author_id, u.username, u.avatar_path, r.content, r.created_at, r.updated_at FROM forum_replies r JOIN users u ON r.author_id = u.id WHERE r.post_id = ?1 ORDER BY r.created_at"
    )?;

    let replies = stmt.query_map([post_id], |row| {
        Ok(ForumReply {
            id: row.get(0)?,
            post_id: row.get(1)?,
            author_id: row.get(2)?,
            author_username: row.get(3)?,
            author_avatar: row.get(4)?,
            content: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(replies)
}
