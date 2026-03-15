// Gestion d'une connexion WebSocket individuelle

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::auth::jwt;
use crate::config::Config;
use crate::db::pool::DbPool;
use crate::db::queries::{friends, servers, users};
use crate::ws::events::{ClientEvent, ServerEvent};
use crate::ws::handlers::{handle_event, WsContext};
use crate::ws::hub::Hub;

/// Gère une connexion WebSocket du début à la fin
pub async fn handle_ws_connection(socket: WebSocket, pool: DbPool, hub: Hub, config: Config) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Tâche d'envoi : relaie les messages du hub vers le WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Attend le premier message AUTH
    let mut user_id = String::new();
    let mut username = String::new();

    if let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(event) = serde_json::from_str::<ClientEvent>(&text) {
                match event {
                    ClientEvent::Auth { token } => {
                        match jwt::validate_token(&token, &config.jwt_secret) {
                            Ok(claims) => {
                                user_id = claims.sub.clone();
                                username = claims.username.clone();

                                // Enregistre le client dans le hub
                                hub.register(&user_id, tx.clone()).await;

                                // Charge les données initiales
                                let conn = match pool.get() {
                                    Ok(c) => c,
                                    Err(_) => {
                                        let _ = tx.send(ServerEvent::AuthError {
                                            message: "Erreur de base de données".to_string(),
                                        }.to_json());
                                        return;
                                    }
                                };

                                let user = users::get_user_by_id(&conn, &user_id).ok();
                                let user_servers = servers::get_user_servers(&conn, &user_id).unwrap_or_default();
                                let friend_list = friends::get_friends(&conn, &user_id).unwrap_or_default();

                                // Rejoint les rooms de ses serveurs
                                let server_ids: Vec<String> = user_servers.iter().map(|s| s.id.clone()).collect();
                                hub.join_server_rooms(&user_id, &server_ids).await;

                                // Envoie AUTH_OK
                                let auth_ok = ServerEvent::AuthOk {
                                    user: serde_json::to_value(&user).unwrap_or_default(),
                                    servers: serde_json::to_value(&user_servers).unwrap_or_default(),
                                    friends: serde_json::to_value(&friend_list).unwrap_or_default(),
                                };
                                let _ = tx.send(auth_ok.to_json());

                                tracing::info!("Utilisateur {} connecté via WebSocket", username);
                            }
                            Err(_) => {
                                let _ = tx.send(ServerEvent::AuthError {
                                    message: "Token invalide".to_string(),
                                }.to_json());
                                return;
                            }
                        }
                    }
                    _ => {
                        let _ = tx.send(ServerEvent::AuthError {
                            message: "Authentification requise".to_string(),
                        }.to_json());
                        return;
                    }
                }
            }
        }
    }

    if user_id.is_empty() {
        return;
    }

    // Contexte pour les handlers
    let ctx = WsContext {
        user_id: user_id.clone(),
        username: username.clone(),
        pool: pool.clone(),
        hub: hub.clone(),
    };

    // Boucle principale de réception des messages
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<ClientEvent>(&text) {
                    Ok(event) => {
                        if let Some(response) = handle_event(&ctx, event).await {
                            let _ = tx.send(response.to_json());
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(ServerEvent::Error {
                            code: 400,
                            message: format!("Événement invalide : {}", e),
                        }.to_json());
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Nettoyage à la déconnexion
    hub.unregister(&user_id).await;
    send_task.abort();

    tracing::info!("Utilisateur {} déconnecté", username);
}
