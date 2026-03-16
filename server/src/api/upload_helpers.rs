// Helpers pour les uploads — lecture limitée et validation de type
use axum::extract::multipart::Field;
use crate::error::AppError;

/// Taille maximale pour les images (10 Mo)
const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024;

/// Types MIME autorisés pour les images
const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
];

/// Lit un champ multipart en limitant la taille pour éviter un OOM
pub async fn read_limited_field(field: &mut Field<'_>, max_size: usize) -> Result<Vec<u8>, AppError> {
    use futures_util::TryStreamExt;

    let mut data = Vec::new();
    let mut stream = field.into_stream();

    while let Some(chunk) = stream.try_next().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if data.len() + chunk.len() > max_size {
            return Err(AppError::BadRequest(format!(
                "Fichier trop volumineux (max {} Mo)",
                max_size / 1024 / 1024
            )));
        }
        data.extend_from_slice(&chunk);
    }

    Ok(data)
}

/// Valide qu'un Content-Type correspond à une image autorisée
pub fn validate_image_content_type(content_type: Option<&str>) -> Result<(), AppError> {
    match content_type {
        Some(ct) if ALLOWED_IMAGE_TYPES.contains(&ct) => Ok(()),
        Some(ct) => Err(AppError::BadRequest(format!(
            "Type de fichier non autorisé : {}. Types acceptés : PNG, JPEG, GIF, WebP",
            ct
        ))),
        None => Err(AppError::BadRequest(
            "Type de fichier non spécifié. Types acceptés : PNG, JPEG, GIF, WebP".to_string()
        )),
    }
}

/// Lit et valide un champ image (Content-Type + taille limitée)
pub async fn read_image_field(field: &mut Field<'_>) -> Result<Vec<u8>, AppError> {
    let content_type = field.content_type().map(|s| s.to_string());
    validate_image_content_type(content_type.as_deref())?;
    read_limited_field(field, MAX_IMAGE_SIZE).await
}
