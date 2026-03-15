// Gestionnaires d'événements WebSocket — logique métier pour chaque type d'événement

use crate::db::pool::DbPool;
use crate::db::queries::{channels, forum, friends, messages, servers};
use crate::ws::events::{ClientEvent, ServerEvent};
use crate::ws::hub::Hub;

/// Contexte d'un utilisateur authentifié via WebSocket
pub struct WsContext {
    pub user_id: String,
    pub username: String,
    pub pool: DbPool,
    pub hub: Hub,
}

/// Traite un événement client et retourne un éventuel événement de réponse
pub async fn handle_event(ctx: &WsContext, event: ClientEvent) -> Option<ServerEvent> {
    match event {
        ClientEvent::MessageSend {
            channel_id,
            content,
            file,
            reply_to_id,
        } => handle_message_send(ctx, &channel_id, &content, file.as_deref(), reply_to_id.as_deref()).await,

        ClientEvent::MessageEdit {
            message_id,
            content,
        } => handle_message_edit(ctx, &message_id, &content).await,

        ClientEvent::MessageDelete { message_id } => handle_message_delete(ctx, &message_id).await,

        ClientEvent::ReactionAdd { message_id, emoji } => {
            handle_reaction_add(ctx, &message_id, &emoji).await
        }

        ClientEvent::ReactionRemove { message_id, emoji } => {
            handle_reaction_remove(ctx, &message_id, &emoji).await
        }

        ClientEvent::DmSend {
            receiver_id,
            content,
            file,
        } => handle_dm_send(ctx, &receiver_id, &content, file.as_deref()).await,

        ClientEvent::VoiceJoin { channel_id } => handle_voice_join(ctx, &channel_id).await,

        ClientEvent::VoiceLeave { channel_id } => handle_voice_leave(ctx, &channel_id).await,

        ClientEvent::WebrtcSignal {
            target_user_id,
            signal_data,
        } => handle_webrtc_signal(ctx, &target_user_id, signal_data).await,

        ClientEvent::TypingStart { channel_id } => handle_typing(ctx, &channel_id).await,

        ClientEvent::ChannelMessagesLoad {
            channel_id,
            before_id,
            limit,
        } => handle_load_messages(ctx, &channel_id, before_id.as_deref(), limit).await,

        ClientEvent::ForumPostCreate {
            channel_id,
            title,
            content,
        } => handle_forum_post(ctx, &channel_id, &title, &content).await,

        ClientEvent::ForumReplyCreate { post_id, content } => {
            handle_forum_reply(ctx, &post_id, &content).await
        }

        // AUTH est géré séparément dans connection.rs
        ClientEvent::Auth { .. } => None,
    }
}

/// Gère l'envoi d'un message
async fn handle_message_send(
    ctx: &WsContext,
    channel_id: &str,
    content: &str,
    file: Option<&str>,
    reply_to_id: Option<&str>,
) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return Some(ServerEvent::Error { code: 500, message: "Erreur de base de données".to_string() }),
    };

    let msg = match messages::create_message(&conn, channel_id, &ctx.user_id, content, file, reply_to_id) {
        Ok(m) => m,
        Err(e) => return Some(ServerEvent::Error { code: 400, message: e.to_string() }),
    };

    // Trouve le serveur du salon pour diffuser
    if let Ok(server_id) = channels::get_channel_server_id(&conn, channel_id) {
        let event = ServerEvent::MessageNew {
            message: serde_json::to_value(&msg).unwrap_or_default(),
        };
        ctx.hub.broadcast_to_server(&server_id, &event, None).await;
    }

    None // Pas de réponse directe — le broadcast inclut l'émetteur
}

/// Gère la modification d'un message
async fn handle_message_edit(ctx: &WsContext, message_id: &str, content: &str) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return Some(ServerEvent::Error { code: 500, message: "Erreur DB".to_string() }),
    };

    // Vérifie que l'auteur est bien l'utilisateur
    match messages::get_message_author_id(&conn, message_id) {
        Ok(author) if author == ctx.user_id => {}
        _ => return Some(ServerEvent::Error { code: 403, message: "Non autorisé".to_string() }),
    }

    let msg = match messages::edit_message(&conn, message_id, content) {
        Ok(m) => m,
        Err(e) => return Some(ServerEvent::Error { code: 400, message: e.to_string() }),
    };

    if let Ok(server_id) = channels::get_channel_server_id(&conn, &msg.channel_id) {
        let event = ServerEvent::MessageUpdated {
            message: serde_json::to_value(&msg).unwrap_or_default(),
        };
        ctx.hub.broadcast_to_server(&server_id, &event, None).await;
    }

    None
}

/// Gère la suppression d'un message
async fn handle_message_delete(ctx: &WsContext, message_id: &str) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return Some(ServerEvent::Error { code: 500, message: "Erreur DB".to_string() }),
    };

    let channel_id = match messages::get_message_channel_id(&conn, message_id) {
        Ok(cid) => cid,
        Err(_) => return Some(ServerEvent::Error { code: 404, message: "Message introuvable".to_string() }),
    };

    match messages::get_message_author_id(&conn, message_id) {
        Ok(author) if author == ctx.user_id => {}
        _ => return Some(ServerEvent::Error { code: 403, message: "Non autorisé".to_string() }),
    }

    if let Err(e) = messages::delete_message(&conn, message_id) {
        return Some(ServerEvent::Error { code: 400, message: e.to_string() });
    }

    if let Ok(server_id) = channels::get_channel_server_id(&conn, &channel_id) {
        let event = ServerEvent::MessageDeleted {
            message_id: message_id.to_string(),
            channel_id: channel_id.clone(),
        };
        ctx.hub.broadcast_to_server(&server_id, &event, None).await;
    }

    None
}

/// Gère l'ajout d'une réaction
async fn handle_reaction_add(ctx: &WsContext, message_id: &str, emoji: &str) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return Some(ServerEvent::Error { code: 500, message: "Erreur DB".to_string() }),
    };

    if let Err(e) = messages::add_reaction(&conn, message_id, &ctx.user_id, emoji) {
        return Some(ServerEvent::Error { code: 400, message: e.to_string() });
    }

    if let Ok(channel_id) = messages::get_message_channel_id(&conn, message_id) {
        if let Ok(server_id) = channels::get_channel_server_id(&conn, &channel_id) {
            let event = ServerEvent::ReactionAdded {
                message_id: message_id.to_string(),
                user_id: ctx.user_id.clone(),
                emoji: emoji.to_string(),
            };
            ctx.hub.broadcast_to_server(&server_id, &event, None).await;
        }
    }

    None
}

/// Gère la suppression d'une réaction
async fn handle_reaction_remove(ctx: &WsContext, message_id: &str, emoji: &str) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return Some(ServerEvent::Error { code: 500, message: "Erreur DB".to_string() }),
    };

    if let Err(e) = messages::remove_reaction(&conn, message_id, &ctx.user_id, emoji) {
        return Some(ServerEvent::Error { code: 400, message: e.to_string() });
    }

    if let Ok(channel_id) = messages::get_message_channel_id(&conn, message_id) {
        if let Ok(server_id) = channels::get_channel_server_id(&conn, &channel_id) {
            let event = ServerEvent::ReactionRemoved {
                message_id: message_id.to_string(),
                user_id: ctx.user_id.clone(),
                emoji: emoji.to_string(),
            };
            ctx.hub.broadcast_to_server(&server_id, &event, None).await;
        }
    }

    None
}

/// Gère l'envoi d'un DM
async fn handle_dm_send(ctx: &WsContext, receiver_id: &str, content: &str, file: Option<&str>) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return Some(ServerEvent::Error { code: 500, message: "Erreur DB".to_string() }),
    };

    let dm = match messages::create_dm(&conn, &ctx.user_id, receiver_id, content, file) {
        Ok(d) => d,
        Err(e) => return Some(ServerEvent::Error { code: 400, message: e.to_string() }),
    };

    let event = ServerEvent::DmNew {
        direct_message: serde_json::to_value(&dm).unwrap_or_default(),
    };

    // Envoie au destinataire
    ctx.hub.send_to_user(receiver_id, &event).await;
    // Envoie aussi à l'émetteur
    ctx.hub.send_to_user(&ctx.user_id, &event).await;

    None
}

/// Gère la connexion à un salon vocal
async fn handle_voice_join(ctx: &WsContext, channel_id: &str) -> Option<ServerEvent> {
    ctx.hub.join_voice(channel_id, &ctx.user_id).await;

    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return None,
    };

    if let Ok(server_id) = channels::get_channel_server_id(&conn, channel_id) {
        let event = ServerEvent::VoiceUserJoined {
            channel_id: channel_id.to_string(),
            user_id: ctx.user_id.clone(),
        };
        ctx.hub.broadcast_to_server(&server_id, &event, None).await;
    }

    None
}

/// Gère la déconnexion d'un salon vocal
async fn handle_voice_leave(ctx: &WsContext, channel_id: &str) -> Option<ServerEvent> {
    ctx.hub.leave_voice(channel_id, &ctx.user_id).await;

    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return None,
    };

    if let Ok(server_id) = channels::get_channel_server_id(&conn, channel_id) {
        let event = ServerEvent::VoiceUserLeft {
            channel_id: channel_id.to_string(),
            user_id: ctx.user_id.clone(),
        };
        ctx.hub.broadcast_to_server(&server_id, &event, None).await;
    }

    None
}

/// Gère le relai de signaling WebRTC
async fn handle_webrtc_signal(ctx: &WsContext, target_user_id: &str, signal_data: serde_json::Value) -> Option<ServerEvent> {
    let event = ServerEvent::WebrtcSignal {
        from_user_id: ctx.user_id.clone(),
        signal_data,
    };
    ctx.hub.send_to_user(target_user_id, &event).await;
    None
}

/// Gère l'indicateur de frappe
async fn handle_typing(ctx: &WsContext, channel_id: &str) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return None,
    };

    if let Ok(server_id) = channels::get_channel_server_id(&conn, channel_id) {
        let event = ServerEvent::Typing {
            channel_id: channel_id.to_string(),
            user_id: ctx.user_id.clone(),
        };
        ctx.hub
            .broadcast_to_server(&server_id, &event, Some(&ctx.user_id))
            .await;
    }

    None
}

/// Gère le chargement des messages d'un salon
async fn handle_load_messages(
    ctx: &WsContext,
    channel_id: &str,
    before_id: Option<&str>,
    limit: Option<i64>,
) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return Some(ServerEvent::Error { code: 500, message: "Erreur DB".to_string() }),
    };

    let limit = limit.unwrap_or(50).min(100);
    match messages::get_channel_messages(&conn, channel_id, before_id, limit) {
        Ok(msgs) => Some(ServerEvent::ChannelMessages {
            channel_id: channel_id.to_string(),
            messages: serde_json::to_value(&msgs).unwrap_or_default(),
        }),
        Err(e) => Some(ServerEvent::Error { code: 400, message: e.to_string() }),
    }
}

/// Gère la création d'un post de forum
async fn handle_forum_post(ctx: &WsContext, channel_id: &str, title: &str, content: &str) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return Some(ServerEvent::Error { code: 500, message: "Erreur DB".to_string() }),
    };

    let post = match forum::create_post(&conn, channel_id, &ctx.user_id, title, content) {
        Ok(p) => p,
        Err(e) => return Some(ServerEvent::Error { code: 400, message: e.to_string() }),
    };

    if let Ok(server_id) = channels::get_channel_server_id(&conn, channel_id) {
        let event = ServerEvent::ForumPostNew {
            post: serde_json::to_value(&post).unwrap_or_default(),
        };
        ctx.hub.broadcast_to_server(&server_id, &event, None).await;
    }

    None
}

/// Gère la création d'une réponse de forum
async fn handle_forum_reply(ctx: &WsContext, post_id: &str, content: &str) -> Option<ServerEvent> {
    let conn = match ctx.pool.get() {
        Ok(c) => c,
        Err(_) => return Some(ServerEvent::Error { code: 500, message: "Erreur DB".to_string() }),
    };

    let reply = match forum::create_reply(&conn, post_id, &ctx.user_id, content) {
        Ok(r) => r,
        Err(e) => return Some(ServerEvent::Error { code: 400, message: e.to_string() }),
    };

    // Récupère le channel_id du post pour le broadcast
    let post = match forum::get_post(&conn, post_id) {
        Ok(p) => p,
        Err(_) => return None,
    };

    if let Ok(server_id) = channels::get_channel_server_id(&conn, &post.channel_id) {
        let event = ServerEvent::ForumReplyNew {
            reply: serde_json::to_value(&reply).unwrap_or_default(),
        };
        ctx.hub.broadcast_to_server(&server_id, &event, None).await;
    }

    None
}
