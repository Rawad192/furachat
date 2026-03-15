// Configuration du serveur FuraChat
// Charge les paramètres depuis les variables d'environnement ou utilise des valeurs par défaut

use std::env;
use std::path::PathBuf;

/// Configuration globale du serveur
#[derive(Debug, Clone)]
pub struct Config {
    /// Port d'écoute HTTP/WS
    pub port: u16,
    /// Adresse d'écoute
    pub host: String,
    /// Chemin vers la base de données SQLite
    pub database_path: PathBuf,
    /// Clé secrète pour signer les JWT
    pub jwt_secret: String,
    /// Durée de validité des JWT en secondes (défaut : 7 jours)
    pub jwt_expiration_seconds: i64,
    /// Répertoire racine pour les fichiers uploadés
    pub data_dir: PathBuf,
    /// Seuil d'espace disque libre en octets pour déclencher l'archivage (défaut : 500 Mo)
    pub archive_threshold_bytes: u64,
    /// Taille maximale d'upload en octets (défaut : 50 Mo)
    pub max_upload_size: usize,
}

impl Config {
    /// Charge la configuration depuis les variables d'environnement
    pub fn from_env() -> Self {
        let data_dir = PathBuf::from(
            env::var("FURACHAT_DATA_DIR").unwrap_or_else(|_| "data".to_string()),
        );

        Self {
            port: env::var("FURACHAT_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            host: env::var("FURACHAT_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            database_path: data_dir.join("furachat.db"),
            jwt_secret: env::var("FURACHAT_JWT_SECRET")
                .unwrap_or_else(|_| "furachat-dev-secret-change-in-production".to_string()),
            jwt_expiration_seconds: env::var("FURACHAT_JWT_EXPIRATION")
                .ok()
                .and_then(|e| e.parse().ok())
                .unwrap_or(604800), // 7 jours
            data_dir,
            archive_threshold_bytes: env::var("FURACHAT_ARCHIVE_THRESHOLD")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(500 * 1024 * 1024), // 500 Mo
            max_upload_size: env::var("FURACHAT_MAX_UPLOAD")
                .ok()
                .and_then(|m| m.parse().ok())
                .unwrap_or(50 * 1024 * 1024), // 50 Mo
        }
    }

    /// Crée les répertoires de données nécessaires
    pub fn ensure_data_dirs(&self) -> std::io::Result<()> {
        let dirs = ["avatars", "banners", "files", "stickers", "badges", "archives"];
        for dir in &dirs {
            std::fs::create_dir_all(self.data_dir.join(dir))?;
        }
        Ok(())
    }
}
