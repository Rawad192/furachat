// Point d'entrée du serveur FuraChat
// Configure et lance le serveur Axum avec toutes les routes

mod api;
mod auth;
mod config;
mod db;
mod error;
mod models;
mod storage;
mod webrtc;
mod ws;

use auth::middleware::JwtSecret;
use config::Config;
use db::pool::DbPool;
use tower_http::cors::{Any, CorsLayer};
use ws::hub::Hub;

/// État partagé de l'application, accessible dans tous les handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Config,
    pub hub: Hub,
}

#[tokio::main]
async fn main() {
    // Initialise le système de logs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Charge la configuration
    let config = Config::from_env();
    tracing::info!("Démarrage de FuraChat sur {}:{}", config.host, config.port);

    // Crée les répertoires de données
    if let Err(e) = config.ensure_data_dirs() {
        tracing::error!("Impossible de créer les répertoires de données : {}", e);
        std::process::exit(1);
    }

    // Initialise la base de données
    let pool = match db::pool::create_pool(&config.database_path) {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Impossible de créer le pool de connexions : {}", e);
            std::process::exit(1);
        }
    };

    // Exécute les migrations
    {
        let conn = pool.get().expect("Impossible d'obtenir une connexion pour les migrations");
        if let Err(e) = db::migrations::run_migrations(&conn) {
            tracing::error!("Erreur lors des migrations : {}", e);
            std::process::exit(1);
        }
    }

    // Initialise le temps de démarrage pour le health check
    api::health_routes::init_start_time();

    // Hub WebSocket central
    let hub = Hub::new();

    // État partagé
    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
        hub: hub.clone(),
    };

    // CORS permissif pour le développement
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Injecte la clé JWT dans toutes les requêtes via une extension
    let jwt_secret = JwtSecret(config.jwt_secret.clone());

    // Route WebSocket
    let ws_pool = pool.clone();
    let ws_hub = hub.clone();
    let ws_config = config.clone();

    // Construit le routeur principal
    let app = api::create_router()
        .route("/ws", axum::routing::get(move |ws: axum::extract::WebSocketUpgrade| {
            let pool = ws_pool.clone();
            let hub = ws_hub.clone();
            let config = ws_config.clone();
            async move {
                ws.on_upgrade(move |socket| {
                    ws::connection::handle_ws_connection(socket, pool, hub, config)
                })
            }
        }))
        .layer(cors)
        .layer(axum::Extension(jwt_secret))
        .with_state(state);

    // Lance la tâche d'archivage périodique
    let archive_pool = pool.clone();
    let archive_config = config.clone();
    tokio::spawn(async move {
        storage::archiver::run_periodic_archiver(archive_pool, archive_config).await;
    });

    // Lance le serveur
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Impossible de se lier à l'adresse");

    tracing::info!("Serveur FuraChat démarré sur {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Erreur du serveur");
}
