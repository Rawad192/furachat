// Archivage automatique — exporte les messages en JSON quand l'espace disque est faible

use chrono::Utc;
use rusqlite::params;
use std::path::PathBuf;

use crate::config::Config;
use crate::db::pool::DbPool;

/// Lance la tâche d'archivage périodique (vérifie toutes les 5 minutes)
pub async fn run_periodic_archiver(pool: DbPool, config: Config) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));

    loop {
        interval.tick().await;

        match check_and_archive(&pool, &config) {
            Ok(archived) => {
                if archived {
                    tracing::info!("Archivage automatique effectué avec succès");
                }
            }
            Err(e) => {
                tracing::error!("Erreur lors de l'archivage : {}", e);
            }
        }
    }
}

/// Vérifie l'espace disque et archive si nécessaire
fn check_and_archive(pool: &DbPool, config: &Config) -> Result<bool, Box<dyn std::error::Error>> {
    // Vérifie l'espace disque disponible
    let available = get_available_disk_space(&config.data_dir)?;

    if available > config.archive_threshold_bytes {
        return Ok(false); // Assez d'espace, pas besoin d'archiver
    }

    tracing::warn!(
        "Espace disque faible ({} Mo restants, seuil : {} Mo). Lancement de l'archivage...",
        available / 1024 / 1024,
        config.archive_threshold_bytes / 1024 / 1024
    );

    let conn = pool.get()?;

    // Exporte tous les messages de tous les serveurs
    let mut archive = serde_json::json!({
        "archived_at": Utc::now().to_rfc3339(),
        "servers": {}
    });

    // Récupère tous les serveurs
    let mut server_stmt = conn.prepare("SELECT id, name FROM servers")?;
    let servers: Vec<(String, String)> = server_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .into_iter()
        .flatten()
        .filter_map(|r| r.ok())
        .collect();

    for (server_id, server_name) in &servers {
        let mut server_data = serde_json::json!({
            "name": server_name,
            "channels": {}
        });

        // Récupère les salons du serveur
        let mut channel_stmt = conn.prepare("SELECT id, name FROM channels WHERE server_id = ?1")?;
        let channels: Vec<(String, String)> = channel_stmt
            .query_map([server_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .into_iter()
            .flatten()
            .filter_map(|r| r.ok())
            .collect();

        for (channel_id, channel_name) in &channels {
            // Récupère tous les messages du salon
            let mut msg_stmt = conn.prepare(
                "SELECT m.id, u.username, m.content, m.file_path, m.created_at FROM messages m JOIN users u ON m.author_id = u.id WHERE m.channel_id = ?1 ORDER BY m.created_at"
            )?;

            let messages: Vec<serde_json::Value> = msg_stmt
                .query_map([channel_id], |row| {
                    let id: String = row.get(0)?;
                    let author: String = row.get(1)?;
                    let content: String = row.get(2)?;
                    let file: Option<String> = row.get(3)?;
                    let created_at: String = row.get(4)?;

                    Ok(serde_json::json!({
                        "id": id,
                        "author": author,
                        "content": content,
                        "file": file,
                        "created_at": created_at
                    }))
                })
                .into_iter()
                .flatten()
                .filter_map(|r| r.ok())
                .collect();

            if !messages.is_empty() {
                server_data["channels"][channel_id] = serde_json::json!({
                    "name": channel_name,
                    "messages": messages
                });
            }
        }

        archive["servers"][server_id] = server_data;
    }

    // Sauvegarde l'archive dans un fichier JSON
    let timestamp = Utc::now().format("%Y-%m-%d_%H%M%S");
    let archive_filename = format!("archive_{}.json", timestamp);
    let archive_path = PathBuf::from(&config.data_dir)
        .join("archives")
        .join(&archive_filename);

    let json_str = serde_json::to_string_pretty(&archive)?;
    std::fs::write(&archive_path, &json_str)?;

    tracing::info!("Archive sauvegardée : {}", archive_path.display());

    // Supprime les messages archivés (garde les messages des dernières 24h)
    let cutoff = (Utc::now() - chrono::Duration::hours(24))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    // Bug 6 fix : supprimer les fichiers attachés avant de supprimer les messages
    let mut file_stmt = conn.prepare(
        "SELECT file_path FROM messages WHERE created_at < ?1 AND file_path IS NOT NULL"
    )?;
    let orphan_files: Vec<String> = file_stmt
        .query_map(params![&cutoff], |row| row.get(0))
        .into_iter()
        .flatten()
        .filter_map(|r| r.ok())
        .collect();

    for file_path in &orphan_files {
        let full_path = PathBuf::from(&config.data_dir).join(file_path);
        if full_path.exists() {
            if let Err(e) = std::fs::remove_file(&full_path) {
                tracing::warn!("Impossible de supprimer le fichier orphelin {} : {}", full_path.display(), e);
            } else {
                tracing::debug!("Fichier orphelin supprimé : {}", full_path.display());
            }
        }
    }

    if !orphan_files.is_empty() {
        tracing::info!("{} fichiers orphelins supprimés", orphan_files.len());
    }

    let deleted = conn.execute(
        "DELETE FROM messages WHERE created_at < ?1",
        params![&cutoff],
    )?;

    tracing::info!("{} messages supprimés après archivage", deleted);

    Ok(true)
}

/// Récupère l'espace disque disponible en octets
fn get_available_disk_space(path: &std::path::Path) -> Result<u64, Box<dyn std::error::Error>> {
    // Unix : utilise statvfs
    #[cfg(unix)]
    {
        return nix_statvfs(path);
    }

    // Windows : utilise GetDiskFreeSpaceExW via FFI
    #[cfg(windows)]
    {
        return windows_disk_space(path);
    }

    // Autres plateformes : retourne une valeur sûre qui déclenche l'archivage plutôt que de l'ignorer
    #[cfg(not(any(unix, windows)))]
    {
        tracing::warn!("Détection d'espace disque non supportée sur cette plateforme");
        Ok(u64::MAX)
    }
}

/// Wrapper pour statvfs sur Unix
#[cfg(unix)]
fn nix_statvfs(path: &std::path::Path) -> Result<u64, Box<dyn std::error::Error>> {
    use std::ffi::CString;

    let path_str = path.to_str().ok_or("Chemin invalide")?;
    let c_path = CString::new(path_str)?;

    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
            Ok(stat.f_bavail as u64 * stat.f_bsize as u64)
        } else {
            Err("Erreur statvfs".into())
        }
    }
}

/// Récupère l'espace disque disponible sur Windows via kernel32
#[cfg(windows)]
fn windows_disk_space(path: &std::path::Path) -> Result<u64, Box<dyn std::error::Error>> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Résoudre le chemin absolu et récupérer la racine du disque
    let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let root = abs_path
        .ancestors()
        .last()
        .unwrap_or(path)
        .to_path_buf();
    let root_str = root.to_str().unwrap_or("C:\\");

    // Convertir en wide string pour l'API Windows
    let wide: Vec<u16> = OsStr::new(root_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;

    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    let result = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        )
    };

    if result != 0 {
        Ok(free_bytes_available)
    } else {
        Err("Erreur GetDiskFreeSpaceExW".into())
    }
}
