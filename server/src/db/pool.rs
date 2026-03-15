// Pool de connexions SQLite avec r2d2
// Fournit un accès thread-safe à la base de données

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

/// Type alias pour le pool de connexions SQLite
pub type DbPool = Pool<SqliteConnectionManager>;

/// Crée un nouveau pool de connexions SQLite
pub fn create_pool(database_path: &Path) -> Result<DbPool, r2d2::Error> {
    let manager = SqliteConnectionManager::file(database_path);
    let pool = Pool::builder()
        .max_size(10)
        .build(manager)?;

    // Active les clés étrangères et le mode WAL pour de meilleures performances
    let conn = pool.get()?;
    if let Err(e) = conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;
         PRAGMA busy_timeout=5000;"
    ) {
        tracing::error!("Erreur lors de la configuration SQLite : {:?}", e);
    }

    Ok(pool)
}
