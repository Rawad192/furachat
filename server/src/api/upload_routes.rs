// Routes upload — upload et téléchargement de fichiers

use axum::{
    extract::{Multipart, Path, State},
    routing::{get, post},
    Json, Router,
};
use std::path::PathBuf;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::AppState;

/// Routes upload
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/upload", post(upload_file))
        .route("/api/files/{*path}", get(serve_file))
}

/// POST /api/upload — upload un fichier
async fn upload_file(
    State(state): State<AppState>,
    _auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if field.name() == Some("file") {
            let original_name = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "file".to_string());

            let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;

            // Limite la taille
            if data.len() > state.config.max_upload_size {
                return Err(AppError::BadRequest(format!(
                    "Fichier trop volumineux (max {} Mo)",
                    state.config.max_upload_size / 1024 / 1024
                )));
            }

            // Génère un nom unique pour éviter les collisions
            let ext = original_name
                .rsplit('.')
                .next()
                .unwrap_or("bin");
            let filename = format!("{}_{}.{}", Uuid::new_v4(), sanitize_filename(&original_name), ext);
            let path = PathBuf::from(&state.config.data_dir).join("files").join(&filename);

            tokio::fs::write(&path, &data).await?;

            let file_path = format!("files/{}", filename);
            return Ok(Json(serde_json::json!({
                "file_path": file_path,
                "file_name": original_name,
                "file_size": data.len()
            })));
        }
    }

    Err(AppError::BadRequest("Aucun fichier fourni".to_string()))
}

/// GET /api/files/*path — sert un fichier statique depuis le répertoire data/
async fn serve_file(
    State(state): State<AppState>,
    Path(file_path): Path<String>,
) -> Result<axum::response::Response, AppError> {
    let full_path = PathBuf::from(&state.config.data_dir).join(&file_path);

    if !full_path.exists() {
        return Err(AppError::NotFound("Fichier introuvable".to_string()));
    }

    // Vérifie que le chemin ne remonte pas au-dessus de data/
    let canonical = full_path.canonicalize()?;
    let data_dir = PathBuf::from(&state.config.data_dir).canonicalize()?;
    if !canonical.starts_with(&data_dir) {
        return Err(AppError::Forbidden("Accès refusé".to_string()));
    }

    let data = tokio::fs::read(&full_path).await?;

    // Détecte le type MIME basique
    let content_type = match file_path.rsplit('.').next() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    };

    Ok(axum::response::Response::builder()
        .header("Content-Type", content_type)
        .header("Cache-Control", "public, max-age=86400")
        .body(axum::body::Body::from(data))
        .unwrap_or_else(|_| {
            axum::response::Response::new(axum::body::Body::empty())
        }))
}

/// Nettoie un nom de fichier pour le stockage
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .take(50)
        .collect()
}
