// Modèle salon (channel) et catégorie

use serde::{Deserialize, Serialize};

/// Types de salons possibles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Text,
    Voice,
    Video,
    Screen,
    Forum,
    Announcement,
    Nsfw,
}

impl ChannelType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::Voice => "voice",
            Self::Video => "video",
            Self::Screen => "screen",
            Self::Forum => "forum",
            Self::Announcement => "announcement",
            Self::Nsfw => "nsfw",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(Self::Text),
            "voice" => Some(Self::Voice),
            "video" => Some(Self::Video),
            "screen" => Some(Self::Screen),
            "forum" => Some(Self::Forum),
            "announcement" => Some(Self::Announcement),
            "nsfw" => Some(Self::Nsfw),
            _ => None,
        }
    }
}

/// Salon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub server_id: String,
    pub category_id: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub topic: String,
    pub position: i32,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Catégorie de salons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub position: i32,
    pub created_at: String,
}

/// Corps de requête pour créer un salon
#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub category_id: Option<String>,
}

/// Corps de requête pour modifier un salon
#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub is_archived: Option<bool>,
}

/// Corps de requête pour créer une catégorie
#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
}

/// Corps de requête pour modifier une catégorie
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub position: Option<i32>,
}
