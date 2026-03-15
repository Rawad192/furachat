// Routes rôles — CRUD rôles, attribution aux membres

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, patch, post},
    Json, Router,
};

use crate::auth::middleware::AuthUser;
use crate::db::queries::{roles, servers};
use crate::error::AppError;
use crate::models::role::{CreateRoleRequest, Role, UpdateRoleRequest};
use crate::AppState;

/// Routes rôles
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/servers/{id}/roles", post(create_role))
        .route("/api/roles/{id}", patch(update_role).delete(delete_role))
        .route(
            "/api/servers/{sid}/members/{uid}/roles/{rid}",
            post(assign_role).delete(remove_role),
        )
}

/// POST /api/servers/:id/roles — crée un rôle
async fn create_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(server_id): Path<String>,
    Json(body): Json<CreateRoleRequest>,
) -> Result<Json<Role>, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut créer des rôles".to_string()));
    }

    let color = body.color.as_deref().unwrap_or("#ffffff");
    let permissions = body.permissions.as_ref().cloned().unwrap_or(serde_json::json!({}));

    let role = roles::create_role(&conn, &server_id, &body.name, color, &permissions)?;
    Ok(Json(role))
}

/// PATCH /api/roles/:id — modifie un rôle
async fn update_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(role_id): Path<String>,
    Json(body): Json<UpdateRoleRequest>,
) -> Result<Json<Role>, AppError> {
    let conn = state.pool.get()?;

    let server_id = roles::get_role_server_id(&conn, &role_id)?;
    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut modifier les rôles".to_string()));
    }

    let role = roles::update_role(
        &conn,
        &role_id,
        body.name.as_deref(),
        body.color.as_deref(),
        body.permissions.as_ref(),
        body.position,
    )?;

    Ok(Json(role))
}

/// DELETE /api/roles/:id — supprime un rôle
async fn delete_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(role_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    let server_id = roles::get_role_server_id(&conn, &role_id)?;
    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut supprimer des rôles".to_string()));
    }

    roles::delete_role(&conn, &role_id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/servers/:sid/members/:uid/roles/:rid — assigne un rôle
async fn assign_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((server_id, user_id, role_id)): Path<(String, String, String)>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut attribuer des rôles".to_string()));
    }

    roles::assign_role(&conn, &server_id, &user_id, &role_id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/servers/:sid/members/:uid/roles/:rid — retire un rôle
async fn remove_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((server_id, user_id, role_id)): Path<(String, String, String)>,
) -> Result<StatusCode, AppError> {
    let conn = state.pool.get()?;

    if !servers::is_owner(&conn, &server_id, &auth.user_id)? {
        return Err(AppError::Forbidden("Seul le propriétaire peut retirer des rôles".to_string()));
    }

    roles::remove_role(&conn, &server_id, &user_id, &role_id)?;
    Ok(StatusCode::NO_CONTENT)
}
