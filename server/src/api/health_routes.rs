// Route de santé — vérifie que le serveur est en ligne

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::time::Instant;

use crate::AppState;

/// Point de départ du serveur (stocké globalement pour calculer l'uptime)
static mut START_TIME: Option<Instant> = None;

/// Initialise le temps de démarrage
pub fn init_start_time() {
    unsafe {
        START_TIME = Some(Instant::now());
    }
}

/// Routes de santé
pub fn routes() -> Router<AppState> {
    Router::new().route("/api/health", get(health_check))
}

/// GET /api/health — retourne l'état du serveur
async fn health_check() -> Json<Value> {
    let uptime = unsafe {
        START_TIME
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0)
    };

    Json(json!({
        "name": "FuraChat",
        "version": "1.0.0",
        "status": "ok",
        "uptime": uptime
    }))
}
