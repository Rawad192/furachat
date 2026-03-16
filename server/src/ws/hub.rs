// Hub central WebSocket — gère les connexions, les rooms et la diffusion des messages

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::events::ServerEvent;

/// Message envoyé via le hub
pub type HubMessage = String;

/// Émetteur pour un client connecté
pub type ClientSender = mpsc::UnboundedSender<HubMessage>;

/// Hub central qui maintient la carte des connexions et des rooms
#[derive(Clone)]
pub struct Hub {
    /// Map user_id -> sender (un seul client par utilisateur pour simplifier)
    clients: Arc<RwLock<HashMap<String, ClientSender>>>,
    /// Map server_id -> set de user_ids (membres connectés)
    server_rooms: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Map channel_id -> set de user_ids (utilisateurs dans un salon vocal)
    voice_rooms: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl Hub {
    /// Crée un nouveau hub
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            server_rooms: Arc::new(RwLock::new(HashMap::new())),
            voice_rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Enregistre un client connecté
    pub async fn register(&self, user_id: &str, sender: ClientSender) {
        self.clients.write().await.insert(user_id.to_string(), sender);
    }

    /// Désenregistre un client
    pub async fn unregister(&self, user_id: &str) {
        self.clients.write().await.remove(user_id);

        // Retire de toutes les rooms serveur
        let mut rooms = self.server_rooms.write().await;
        for members in rooms.values_mut() {
            members.remove(user_id);
        }

        // Retire de toutes les rooms vocales
        let mut voice = self.voice_rooms.write().await;
        for members in voice.values_mut() {
            members.remove(user_id);
        }
    }

    /// Ajoute un utilisateur aux rooms de ses serveurs
    pub async fn join_server_rooms(&self, user_id: &str, server_ids: &[String]) {
        let mut rooms = self.server_rooms.write().await;
        for server_id in server_ids {
            rooms
                .entry(server_id.clone())
                .or_default()
                .insert(user_id.to_string());
        }
    }

    /// Envoie un message à un utilisateur spécifique
    pub async fn send_to_user(&self, user_id: &str, event: &ServerEvent) {
        let clients = self.clients.read().await;
        if let Some(sender) = clients.get(user_id) {
            let _ = sender.send(event.to_json());
        }
    }

    /// Diffuse un événement à tous les membres connectés d'un serveur
    pub async fn broadcast_to_server(&self, server_id: &str, event: &ServerEvent, exclude_user: Option<&str>) {
        let rooms = self.server_rooms.read().await;
        let clients = self.clients.read().await;

        if let Some(members) = rooms.get(server_id) {
            let json = event.to_json();
            for user_id in members {
                if exclude_user == Some(user_id.as_str()) {
                    continue;
                }
                if let Some(sender) = clients.get(user_id) {
                    let _ = sender.send(json.clone());
                }
            }
        }
    }

    /// Diffuse un événement aux membres d'un salon vocal
    pub async fn broadcast_to_voice_channel(&self, channel_id: &str, event: &ServerEvent, exclude_user: Option<&str>) {
        let voice = self.voice_rooms.read().await;
        let clients = self.clients.read().await;

        if let Some(members) = voice.get(channel_id) {
            let json = event.to_json();
            for user_id in members {
                if exclude_user == Some(user_id.as_str()) {
                    continue;
                }
                if let Some(sender) = clients.get(user_id) {
                    let _ = sender.send(json.clone());
                }
            }
        }
    }

    /// Ajoute un utilisateur à un salon vocal
    pub async fn join_voice(&self, channel_id: &str, user_id: &str) {
        self.voice_rooms
            .write()
            .await
            .entry(channel_id.to_string())
            .or_default()
            .insert(user_id.to_string());
    }

    /// Retire un utilisateur d'un salon vocal
    pub async fn leave_voice(&self, channel_id: &str, user_id: &str) {
        let mut voice = self.voice_rooms.write().await;
        if let Some(members) = voice.get_mut(channel_id) {
            members.remove(user_id);
            if members.is_empty() {
                voice.remove(channel_id);
            }
        }
    }

    /// Vérifie si un utilisateur est connecté
    pub async fn is_online(&self, user_id: &str) -> bool {
        self.clients.read().await.contains_key(user_id)
    }

    /// Vérifie si deux utilisateurs partagent un salon vocal
    pub async fn share_voice_channel(&self, user_a: &str, user_b: &str) -> bool {
        let voice = self.voice_rooms.read().await;
        for members in voice.values() {
            if members.contains(user_a) && members.contains(user_b) {
                return true;
            }
        }
        false
    }
}
