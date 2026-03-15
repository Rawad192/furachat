// Définition des événements WebSocket — sérialisation et désérialisation

use serde::{Deserialize, Serialize};

/// Événement reçu du client
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientEvent {
    #[serde(rename = "AUTH")]
    Auth { token: String },

    #[serde(rename = "MESSAGE_SEND")]
    MessageSend {
        channel_id: String,
        content: String,
        file: Option<String>,
        reply_to_id: Option<String>,
    },

    #[serde(rename = "MESSAGE_EDIT")]
    MessageEdit {
        message_id: String,
        content: String,
    },

    #[serde(rename = "MESSAGE_DELETE")]
    MessageDelete { message_id: String },

    #[serde(rename = "REACTION_ADD")]
    ReactionAdd {
        message_id: String,
        emoji: String,
    },

    #[serde(rename = "REACTION_REMOVE")]
    ReactionRemove {
        message_id: String,
        emoji: String,
    },

    #[serde(rename = "DM_SEND")]
    DmSend {
        receiver_id: String,
        content: String,
        file: Option<String>,
    },

    #[serde(rename = "VOICE_JOIN")]
    VoiceJoin { channel_id: String },

    #[serde(rename = "VOICE_LEAVE")]
    VoiceLeave { channel_id: String },

    #[serde(rename = "WEBRTC_SIGNAL")]
    WebrtcSignal {
        target_user_id: String,
        signal_data: serde_json::Value,
    },

    #[serde(rename = "TYPING_START")]
    TypingStart { channel_id: String },

    #[serde(rename = "CHANNEL_MESSAGES_LOAD")]
    ChannelMessagesLoad {
        channel_id: String,
        before_id: Option<String>,
        limit: Option<i64>,
    },

    #[serde(rename = "FORUM_POST_CREATE")]
    ForumPostCreate {
        channel_id: String,
        title: String,
        content: String,
    },

    #[serde(rename = "FORUM_REPLY_CREATE")]
    ForumReplyCreate {
        post_id: String,
        content: String,
    },
}

/// Événement envoyé au client
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerEvent {
    #[serde(rename = "AUTH_OK")]
    AuthOk {
        user: serde_json::Value,
        servers: serde_json::Value,
        friends: serde_json::Value,
    },

    #[serde(rename = "AUTH_ERROR")]
    AuthError { message: String },

    #[serde(rename = "MESSAGE_NEW")]
    MessageNew { message: serde_json::Value },

    #[serde(rename = "MESSAGE_UPDATED")]
    MessageUpdated { message: serde_json::Value },

    #[serde(rename = "MESSAGE_DELETED")]
    MessageDeleted {
        message_id: String,
        channel_id: String,
    },

    #[serde(rename = "REACTION_ADDED")]
    ReactionAdded {
        message_id: String,
        user_id: String,
        emoji: String,
    },

    #[serde(rename = "REACTION_REMOVED")]
    ReactionRemoved {
        message_id: String,
        user_id: String,
        emoji: String,
    },

    #[serde(rename = "DM_NEW")]
    DmNew { direct_message: serde_json::Value },

    #[serde(rename = "USER_JOINED_SERVER")]
    UserJoinedServer {
        server_id: String,
        user: serde_json::Value,
    },

    #[serde(rename = "USER_LEFT_SERVER")]
    UserLeftServer {
        server_id: String,
        user_id: String,
    },

    #[serde(rename = "VOICE_USER_JOINED")]
    VoiceUserJoined {
        channel_id: String,
        user_id: String,
    },

    #[serde(rename = "VOICE_USER_LEFT")]
    VoiceUserLeft {
        channel_id: String,
        user_id: String,
    },

    #[serde(rename = "VOICE_SPEAKING")]
    VoiceSpeaking {
        channel_id: String,
        user_id: String,
        speaking: bool,
    },

    #[serde(rename = "WEBRTC_SIGNAL")]
    WebrtcSignal {
        from_user_id: String,
        signal_data: serde_json::Value,
    },

    #[serde(rename = "TYPING")]
    Typing {
        channel_id: String,
        user_id: String,
    },

    #[serde(rename = "CHANNEL_MESSAGES")]
    ChannelMessages {
        channel_id: String,
        messages: serde_json::Value,
    },

    #[serde(rename = "FORUM_POST_NEW")]
    ForumPostNew { post: serde_json::Value },

    #[serde(rename = "FORUM_REPLY_NEW")]
    ForumReplyNew { reply: serde_json::Value },

    #[serde(rename = "ERROR")]
    Error { code: u16, message: String },
}

impl ServerEvent {
    /// Sérialise l'événement en JSON avec un timestamp
    pub fn to_json(&self) -> String {
        let event = serde_json::to_value(self).unwrap_or(serde_json::json!({}));
        let mut obj = event.as_object().cloned().unwrap_or_default();
        obj.insert(
            "timestamp".to_string(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
        );
        serde_json::to_string(&obj).unwrap_or_default()
    }
}
