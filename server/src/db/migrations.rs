// Migrations de la base de données — création de toutes les tables au premier lancement
// Schéma complet de FuraChat

use rusqlite::Connection;

/// Exécute toutes les migrations pour créer le schéma complet
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(SCHEMA)?;
    tracing::info!("Migrations exécutées avec succès");
    Ok(())
}

/// Schéma SQL complet de FuraChat
const SCHEMA: &str = r#"
-- Table des utilisateurs
CREATE TABLE IF NOT EXISTS users (
    id              TEXT PRIMARY KEY,
    username        TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    avatar_path     TEXT,
    banner_path     TEXT,
    bio             TEXT DEFAULT '',
    status_text     TEXT DEFAULT '',
    status_emoji    TEXT DEFAULT '',
    custom_css      TEXT DEFAULT '',
    social_links    TEXT DEFAULT '{}',
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Relations d'amitié (bidirectionnelles, pas de demande)
CREATE TABLE IF NOT EXISTS friends (
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    friend_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, friend_id)
);

-- Messages privés (DM)
CREATE TABLE IF NOT EXISTS direct_messages (
    id              TEXT PRIMARY KEY,
    sender_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content         TEXT NOT NULL,
    file_path       TEXT,
    edited          BOOLEAN DEFAULT FALSE,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_dm_participants ON direct_messages(sender_id, receiver_id);
CREATE INDEX IF NOT EXISTS idx_dm_time ON direct_messages(created_at);

-- Serveurs
CREATE TABLE IF NOT EXISTS servers (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    icon_path       TEXT,
    banner_path     TEXT,
    owner_id        TEXT NOT NULL REFERENCES users(id),
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Membres d'un serveur
CREATE TABLE IF NOT EXISTS server_members (
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    nickname        TEXT,
    joined_at       DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (server_id, user_id)
);

-- Rôles
CREATE TABLE IF NOT EXISTS roles (
    id              TEXT PRIMARY KEY,
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    color           TEXT DEFAULT '#ffffff',
    position        INTEGER NOT NULL DEFAULT 0,
    permissions     TEXT NOT NULL DEFAULT '{}',
    is_default      BOOLEAN DEFAULT FALSE,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_roles_server ON roles(server_id);

-- Attribution des rôles aux membres
CREATE TABLE IF NOT EXISTS member_roles (
    server_id       TEXT NOT NULL,
    user_id         TEXT NOT NULL,
    role_id         TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (server_id, user_id, role_id),
    FOREIGN KEY (server_id, user_id) REFERENCES server_members(server_id, user_id) ON DELETE CASCADE
);

-- Catégories de salons
CREATE TABLE IF NOT EXISTS categories (
    id              TEXT PRIMARY KEY,
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    position        INTEGER NOT NULL DEFAULT 0,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Salons (channels)
CREATE TABLE IF NOT EXISTS channels (
    id              TEXT PRIMARY KEY,
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    category_id     TEXT REFERENCES categories(id) ON DELETE SET NULL,
    name            TEXT NOT NULL,
    type            TEXT NOT NULL CHECK(type IN ('text','voice','video','screen','forum','announcement','nsfw')),
    topic           TEXT DEFAULT '',
    position        INTEGER NOT NULL DEFAULT 0,
    is_archived     BOOLEAN DEFAULT FALSE,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_channels_server ON channels(server_id);

-- Permissions spécifiques d'un salon (override par rôle)
CREATE TABLE IF NOT EXISTS channel_permissions (
    channel_id      TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    role_id         TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    allow           TEXT NOT NULL DEFAULT '{}',
    deny            TEXT NOT NULL DEFAULT '{}',
    PRIMARY KEY (channel_id, role_id)
);

-- Messages dans les salons
CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY,
    channel_id      TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    author_id       TEXT NOT NULL REFERENCES users(id),
    content         TEXT NOT NULL,
    file_path       TEXT,
    reply_to_id     TEXT REFERENCES messages(id) ON DELETE SET NULL,
    edited          BOOLEAN DEFAULT FALSE,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages(channel_id, created_at);
CREATE INDEX IF NOT EXISTS idx_messages_author ON messages(author_id);

-- Réactions sur les messages
CREATE TABLE IF NOT EXISTS reactions (
    message_id      TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji           TEXT NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (message_id, user_id, emoji)
);

-- Posts de forum
CREATE TABLE IF NOT EXISTS forum_posts (
    id              TEXT PRIMARY KEY,
    channel_id      TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    author_id       TEXT NOT NULL REFERENCES users(id),
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_forum_channel ON forum_posts(channel_id, created_at DESC);

-- Réponses aux posts de forum
CREATE TABLE IF NOT EXISTS forum_replies (
    id              TEXT PRIMARY KEY,
    post_id         TEXT NOT NULL REFERENCES forum_posts(id) ON DELETE CASCADE,
    author_id       TEXT NOT NULL REFERENCES users(id),
    content         TEXT NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_forum_replies ON forum_replies(post_id, created_at);

-- Invitations de serveur
CREATE TABLE IF NOT EXISTS invitations (
    id              TEXT PRIMARY KEY,
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    created_by      TEXT NOT NULL REFERENCES users(id),
    code            TEXT NOT NULL UNIQUE,
    expires_at      DATETIME,
    max_uses        INTEGER,
    use_count       INTEGER DEFAULT 0,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_invitations_code ON invitations(code);

-- Stickers personnels
CREATE TABLE IF NOT EXISTS stickers (
    id              TEXT PRIMARY KEY,
    owner_id        TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    file_path       TEXT NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Badges personnalisés
CREATE TABLE IF NOT EXISTS badges (
    id              TEXT PRIMARY KEY,
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    icon_path       TEXT NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Attribution des badges
CREATE TABLE IF NOT EXISTS user_badges (
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    badge_id        TEXT NOT NULL REFERENCES badges(id) ON DELETE CASCADE,
    awarded_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, badge_id)
);

-- Logs de modération
CREATE TABLE IF NOT EXISTS audit_logs (
    id              TEXT PRIMARY KEY,
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    actor_id        TEXT NOT NULL REFERENCES users(id),
    action          TEXT NOT NULL,
    target_type     TEXT,
    target_id       TEXT,
    details         TEXT DEFAULT '{}',
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_audit_server ON audit_logs(server_id, created_at DESC);

-- Bans
CREATE TABLE IF NOT EXISTS bans (
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    banned_by       TEXT NOT NULL REFERENCES users(id),
    reason          TEXT DEFAULT '',
    expires_at      DATETIME,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (server_id, user_id)
);

-- Mutes (timeout)
CREATE TABLE IF NOT EXISTS mutes (
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    muted_by        TEXT NOT NULL REFERENCES users(id),
    reason          TEXT DEFAULT '',
    expires_at      DATETIME,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (server_id, user_id)
);

-- Ordre personnalisé des salons par utilisateur
CREATE TABLE IF NOT EXISTS user_channel_order (
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_id      TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    position        INTEGER NOT NULL,
    PRIMARY KEY (user_id, channel_id)
);
"#;
