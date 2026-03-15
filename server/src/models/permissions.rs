// Modèle permissions — résolution des permissions par rôle et par salon

use serde::{Deserialize, Serialize};

/// Ensemble de permissions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Permissions {
    #[serde(default)]
    pub manage_server: bool,
    #[serde(default)]
    pub manage_channels: bool,
    #[serde(default)]
    pub manage_roles: bool,
    #[serde(default)]
    pub manage_members: bool,
    #[serde(default = "default_true")]
    pub send_messages: bool,
    #[serde(default = "default_true")]
    pub send_files: bool,
    #[serde(default)]
    pub write_announcements: bool,
    #[serde(default = "default_true")]
    pub connect_voice: bool,
    #[serde(default = "default_true")]
    pub speak_voice: bool,
    #[serde(default = "default_true")]
    pub use_video: bool,
    #[serde(default = "default_true")]
    pub share_screen: bool,
    #[serde(default)]
    pub view_audit_log: bool,
}

fn default_true() -> bool {
    true
}

impl Permissions {
    /// Permissions par défaut pour le rôle @everyone
    pub fn default_everyone() -> Self {
        Self {
            manage_server: false,
            manage_channels: false,
            manage_roles: false,
            manage_members: false,
            send_messages: true,
            send_files: true,
            write_announcements: false,
            connect_voice: true,
            speak_voice: true,
            use_video: true,
            share_screen: true,
            view_audit_log: false,
        }
    }

    /// Permissions complètes (pour le propriétaire du serveur)
    pub fn all() -> Self {
        Self {
            manage_server: true,
            manage_channels: true,
            manage_roles: true,
            manage_members: true,
            send_messages: true,
            send_files: true,
            write_announcements: true,
            connect_voice: true,
            speak_voice: true,
            use_video: true,
            share_screen: true,
            view_audit_log: true,
        }
    }

    /// Fusionne les permissions : applique les rôles par ordre de position
    /// puis les overrides de salon (allow/deny)
    pub fn resolve(
        is_owner: bool,
        everyone_perms: &Permissions,
        role_perms: &[Permissions],
        channel_allow: Option<&Permissions>,
        channel_deny: Option<&Permissions>,
    ) -> Self {
        // Le propriétaire a toutes les permissions
        if is_owner {
            return Self::all();
        }

        // Commence avec les permissions @everyone
        let mut result = everyone_perms.clone();

        // Applique les rôles (les permissions accordées par n'importe quel rôle s'additionnent)
        for role in role_perms {
            result.merge_role(role);
        }

        // Applique les overrides de salon
        if let Some(allow) = channel_allow {
            result.apply_allow(allow);
        }
        if let Some(deny) = channel_deny {
            result.apply_deny(deny);
        }

        result
    }

    /// Fusionne les permissions d'un rôle (OR logique)
    fn merge_role(&mut self, other: &Permissions) {
        self.manage_server = self.manage_server || other.manage_server;
        self.manage_channels = self.manage_channels || other.manage_channels;
        self.manage_roles = self.manage_roles || other.manage_roles;
        self.manage_members = self.manage_members || other.manage_members;
        self.send_messages = self.send_messages || other.send_messages;
        self.send_files = self.send_files || other.send_files;
        self.write_announcements = self.write_announcements || other.write_announcements;
        self.connect_voice = self.connect_voice || other.connect_voice;
        self.speak_voice = self.speak_voice || other.speak_voice;
        self.use_video = self.use_video || other.use_video;
        self.share_screen = self.share_screen || other.share_screen;
        self.view_audit_log = self.view_audit_log || other.view_audit_log;
    }

    /// Applique les permissions autorisées (override salon)
    fn apply_allow(&mut self, allow: &Permissions) {
        if allow.manage_server { self.manage_server = true; }
        if allow.manage_channels { self.manage_channels = true; }
        if allow.manage_roles { self.manage_roles = true; }
        if allow.manage_members { self.manage_members = true; }
        if allow.send_messages { self.send_messages = true; }
        if allow.send_files { self.send_files = true; }
        if allow.write_announcements { self.write_announcements = true; }
        if allow.connect_voice { self.connect_voice = true; }
        if allow.speak_voice { self.speak_voice = true; }
        if allow.use_video { self.use_video = true; }
        if allow.share_screen { self.share_screen = true; }
        if allow.view_audit_log { self.view_audit_log = true; }
    }

    /// Applique les permissions refusées (override salon — un deny l'emporte toujours)
    fn apply_deny(&mut self, deny: &Permissions) {
        if deny.manage_server { self.manage_server = false; }
        if deny.manage_channels { self.manage_channels = false; }
        if deny.manage_roles { self.manage_roles = false; }
        if deny.manage_members { self.manage_members = false; }
        if deny.send_messages { self.send_messages = false; }
        if deny.send_files { self.send_files = false; }
        if deny.write_announcements { self.write_announcements = false; }
        if deny.connect_voice { self.connect_voice = false; }
        if deny.speak_voice { self.speak_voice = false; }
        if deny.use_video { self.use_video = false; }
        if deny.share_screen { self.share_screen = false; }
        if deny.view_audit_log { self.view_audit_log = false; }
    }
}
