// Types TypeScript correspondant aux modèles backend

// ── Utilisateur ──────────────────────────────────────────────
export interface User {
  id: string;
  username: string;
  display_name: string | null;
  email: string;
  avatar_url: string | null;
  banner_url: string | null;
  bio: string | null;
  status: string;
  custom_status: string | null;
  created_at: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface UpdateProfileRequest {
  display_name?: string | null;
  bio?: string | null;
  status?: string;
  custom_status?: string | null;
}

// ── Serveur ──────────────────────────────────────────────────
export interface Server {
  id: string;
  name: string;
  description: string | null;
  icon_url: string | null;
  banner_url: string | null;
  owner_id: string;
  created_at: string;
}

export interface ServerMember {
  user_id: string;
  username: string;
  display_name: string | null;
  avatar_url: string | null;
  status: string;
  custom_status: string | null;
  roles: Role[];
  joined_at: string;
}

export interface CreateServerRequest {
  name: string;
  description?: string;
}

export interface UpdateServerRequest {
  name?: string;
  description?: string | null;
}

export interface ServerDetails {
  server: Server;
  channels: Channel[];
  categories: Category[];
  members: ServerMember[];
  roles: Role[];
}

// ── Canal ────────────────────────────────────────────────────
export type ChannelType = 'text' | 'voice' | 'forum' | 'announcement';

export interface Channel {
  id: string;
  server_id: string;
  category_id: string | null;
  name: string;
  channel_type: ChannelType;
  topic: string | null;
  position: number;
  slowmode_seconds: number;
  nsfw: boolean;
  created_at: string;
}

export interface Category {
  id: string;
  server_id: string;
  name: string;
  position: number;
}

export interface CreateChannelRequest {
  name: string;
  channel_type: ChannelType;
  category_id?: string;
  topic?: string;
}

export interface CreateCategoryRequest {
  name: string;
}

// ── Message ──────────────────────────────────────────────────
export interface Message {
  id: string;
  channel_id: string;
  author_id: string;
  author_username: string;
  author_avatar: string | null;
  content: string;
  attachment_url: string | null;
  edited_at: string | null;
  created_at: string;
  reactions: Reaction[];
}

export interface DirectMessage {
  id: string;
  sender_id: string;
  receiver_id: string;
  sender_username: string;
  sender_avatar: string | null;
  content: string;
  attachment_url: string | null;
  created_at: string;
}

export interface Reaction {
  emoji: string;
  count: number;
  users: string[];
}

export interface SendMessageRequest {
  content: string;
  attachment_url?: string;
}

export interface EditMessageRequest {
  content: string;
}

// ── Rôle ─────────────────────────────────────────────────────
export interface Role {
  id: string;
  server_id: string;
  name: string;
  color: string;
  position: number;
  permissions: Permissions;
  is_default: boolean;
  created_at: string;
}

export interface Permissions {
  administrator: boolean;
  manage_server: boolean;
  manage_channels: boolean;
  manage_roles: boolean;
  manage_messages: boolean;
  kick_members: boolean;
  ban_members: boolean;
  send_messages: boolean;
  read_messages: boolean;
  attach_files: boolean;
  use_voice: boolean;
  mute_members: boolean;
  deafen_members: boolean;
  create_invites: boolean;
  manage_stickers: boolean;
}

export interface CreateRoleRequest {
  name: string;
  color?: string;
  permissions?: Partial<Permissions>;
}

export interface UpdateRoleRequest {
  name?: string;
  color?: string;
  position?: number;
  permissions?: Partial<Permissions>;
}

export interface ChannelPermissionOverride {
  role_id: string;
  channel_id: string;
  allow_send: boolean | null;
  allow_read: boolean | null;
  allow_attach: boolean | null;
  allow_manage: boolean | null;
}

// ── Invitation ───────────────────────────────────────────────
export interface Invitation {
  id: string;
  server_id: string;
  creator_id: string;
  code: string;
  max_uses: number | null;
  uses: number;
  expires_at: string | null;
  created_at: string;
}

export interface InviteInfo {
  code: string;
  server_name: string;
  server_icon: string | null;
  member_count: number;
  creator_username: string;
}

export interface CreateInviteRequest {
  max_uses?: number;
  expires_in_hours?: number;
}

// ── Forum ────────────────────────────────────────────────────
export interface ForumPost {
  id: string;
  channel_id: string;
  author_id: string;
  author_username: string;
  author_avatar: string | null;
  title: string;
  content: string;
  pinned: boolean;
  locked: boolean;
  reply_count: number;
  created_at: string;
}

export interface ForumReply {
  id: string;
  post_id: string;
  author_id: string;
  author_username: string;
  author_avatar: string | null;
  content: string;
  created_at: string;
}

export interface CreateForumPostRequest {
  title: string;
  content: string;
}

export interface CreateForumReplyRequest {
  content: string;
}

// ── Sticker & Badge ──────────────────────────────────────────
export interface Sticker {
  id: string;
  server_id: string;
  name: string;
  image_url: string;
  created_at: string;
}

export interface Badge {
  id: string;
  name: string;
  icon_url: string;
  description: string;
  created_at: string;
}

export interface UserBadge {
  badge: Badge;
  awarded_at: string;
}

// ── Ami ──────────────────────────────────────────────────────
export interface Friend {
  user_id: string;
  username: string;
  display_name: string | null;
  avatar_url: string | null;
  status: string;
}

// ── Modération ───────────────────────────────────────────────
export interface AuditLogEntry {
  id: string;
  server_id: string;
  moderator_id: string;
  moderator_username: string;
  action: string;
  target_id: string | null;
  reason: string | null;
  created_at: string;
}

// ── WebSocket Events ─────────────────────────────────────────
export type ClientEventType =
  | 'AUTH'
  | 'MESSAGE_SEND'
  | 'MESSAGE_EDIT'
  | 'MESSAGE_DELETE'
  | 'REACTION_ADD'
  | 'REACTION_REMOVE'
  | 'DM_SEND'
  | 'VOICE_JOIN'
  | 'VOICE_LEAVE'
  | 'WEBRTC_SIGNAL'
  | 'TYPING_START'
  | 'CHANNEL_MESSAGES_LOAD'
  | 'FORUM_POST_CREATE'
  | 'FORUM_REPLY_CREATE';

export type ServerEventType =
  | 'AUTH_OK'
  | 'MESSAGE_NEW'
  | 'MESSAGE_UPDATED'
  | 'MESSAGE_DELETED'
  | 'REACTION_ADDED'
  | 'REACTION_REMOVED'
  | 'DM_NEW'
  | 'VOICE_USER_JOINED'
  | 'VOICE_USER_LEFT'
  | 'WEBRTC_SIGNAL'
  | 'TYPING'
  | 'CHANNEL_MESSAGES'
  | 'FORUM_POST_NEW'
  | 'FORUM_REPLY_NEW'
  | 'ERROR';

export interface WsMessage {
  type: ClientEventType | ServerEventType;
  [key: string]: unknown;
}
