// Service API — wrapper fetch avec authentification JWT
import type {
  AuthResponse,
  LoginRequest,
  RegisterRequest,
  User,
  UpdateProfileRequest,
  Server,
  ServerDetails,
  CreateServerRequest,
  UpdateServerRequest,
  Channel,
  Category,
  CreateChannelRequest,
  CreateCategoryRequest,
  Message,
  Role,
  CreateRoleRequest,
  UpdateRoleRequest,
  Friend,
  Invitation,
  InviteInfo,
  CreateInviteRequest,
  ForumPost,
  ForumReply,
  CreateForumPostRequest,
  CreateForumReplyRequest,
  Sticker,
  Badge,
  AuditLogEntry,
} from '../types';

import { API_BASE_URL } from '../config';

const BASE_URL = `${API_BASE_URL}/api`;

// ── Helpers ──────────────────────────────────────────────────

function getToken(): string | null {
  return localStorage.getItem('token');
}

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const headers: Record<string, string> = {
    ...(options.headers as Record<string, string>),
  };

  const token = getToken();
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  // Ne pas ajouter Content-Type si c'est un FormData (le navigateur le fait)
  if (!(options.body instanceof FormData)) {
    headers['Content-Type'] = 'application/json';
  }

  const res = await fetch(`${BASE_URL}${path}`, {
    ...options,
    headers,
  });

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new ApiError(res.status, body.error || 'Erreur inconnue');
  }

  // 204 No Content
  if (res.status === 204) {
    return undefined as T;
  }

  return res.json();
}

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

function get<T>(path: string): Promise<T> {
  return request<T>(path);
}

function post<T>(path: string, body?: unknown): Promise<T> {
  return request<T>(path, {
    method: 'POST',
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
}

function patch<T>(path: string, body?: unknown): Promise<T> {
  return request<T>(path, {
    method: 'PATCH',
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
}

function del<T>(path: string): Promise<T> {
  return request<T>(path, { method: 'DELETE' });
}

function upload<T>(path: string, formData: FormData): Promise<T> {
  return request<T>(path, {
    method: 'POST',
    body: formData,
  });
}

// ── Auth ─────────────────────────────────────────────────────
export const auth = {
  register: (data: RegisterRequest) => post<AuthResponse>('/auth/register', data),
  login: (data: LoginRequest) => post<AuthResponse>('/auth/login', data),
};

// ── Utilisateur ──────────────────────────────────────────────
export const users = {
  me: () => get<User>('/users/@me'),
  update: (data: UpdateProfileRequest) => patch<User>('/users/@me', data),
  getById: (id: string) => get<User>(`/users/${id}`),
  uploadAvatar: (file: File) => {
    const fd = new FormData();
    fd.append('file', file);
    return upload<{ url: string }>('/users/@me/avatar', fd);
  },
  uploadBanner: (file: File) => {
    const fd = new FormData();
    fd.append('file', file);
    return upload<{ url: string }>('/users/@me/banner', fd);
  },
};

// ── Serveurs ─────────────────────────────────────────────────
export const servers = {
  list: () => get<Server[]>('/servers'),
  create: (data: CreateServerRequest) => post<Server>('/servers', data),
  get: (id: string) => get<ServerDetails>(`/servers/${id}`),
  update: (id: string, data: UpdateServerRequest) => patch<Server>(`/servers/${id}`, data),
  delete: (id: string) => del<void>(`/servers/${id}`),
  leave: (id: string) => post<void>(`/servers/${id}/leave`),
  uploadIcon: (id: string, file: File) => {
    const fd = new FormData();
    fd.append('file', file);
    return upload<{ url: string }>(`/servers/${id}/icon`, fd);
  },
  uploadBanner: (id: string, file: File) => {
    const fd = new FormData();
    fd.append('file', file);
    return upload<{ url: string }>(`/servers/${id}/banner`, fd);
  },
  members: (id: string) => get<import('../types').ServerMember[]>(`/servers/${id}/members`),
};

// ── Canaux ───────────────────────────────────────────────────
export const channels = {
  create: (serverId: string, data: CreateChannelRequest) =>
    post<Channel>(`/servers/${serverId}/channels`, data),
  update: (serverId: string, channelId: string, data: Partial<CreateChannelRequest>) =>
    patch<Channel>(`/servers/${serverId}/channels/${channelId}`, data),
  delete: (serverId: string, channelId: string) =>
    del<void>(`/servers/${serverId}/channels/${channelId}`),
  createCategory: (serverId: string, data: CreateCategoryRequest) =>
    post<Category>(`/servers/${serverId}/categories`, data),
};

// ── Messages ─────────────────────────────────────────────────
export const messages = {
  list: (channelId: string, beforeId?: string) => {
    const params = beforeId ? `?before_id=${beforeId}` : '';
    return get<Message[]>(`/channels/${channelId}/messages${params}`);
  },
};

// ── Rôles ────────────────────────────────────────────────────
export const roles = {
  create: (serverId: string, data: CreateRoleRequest) =>
    post<Role>(`/servers/${serverId}/roles`, data),
  update: (serverId: string, roleId: string, data: UpdateRoleRequest) =>
    patch<Role>(`/servers/${serverId}/roles/${roleId}`, data),
  delete: (serverId: string, roleId: string) =>
    del<void>(`/servers/${serverId}/roles/${roleId}`),
  assign: (serverId: string, memberId: string, roleId: string) =>
    post<void>(`/servers/${serverId}/members/${memberId}/roles/${roleId}`),
  remove: (serverId: string, memberId: string, roleId: string) =>
    del<void>(`/servers/${serverId}/members/${memberId}/roles/${roleId}`),
};

// ── Amis ─────────────────────────────────────────────────────
export const friends = {
  list: () => get<Friend[]>('/friends'),
  add: (username: string) => post<void>('/friends', { username }),
  remove: (userId: string) => del<void>(`/friends/${userId}`),
};

// ── Invitations ──────────────────────────────────────────────
export const invitations = {
  create: (serverId: string, data?: CreateInviteRequest) =>
    post<Invitation>(`/servers/${serverId}/invites`, data),
  getInfo: (code: string) => get<InviteInfo>(`/invites/${code}`),
  join: (code: string) => post<Server>(`/invites/${code}/join`),
};

// ── Forum ────────────────────────────────────────────────────
export const forum = {
  listPosts: (channelId: string) => get<ForumPost[]>(`/channels/${channelId}/forum/posts`),
  createPost: (channelId: string, data: CreateForumPostRequest) =>
    post<ForumPost>(`/channels/${channelId}/forum/posts`, data),
  getReplies: (channelId: string, postId: string) =>
    get<ForumReply[]>(`/channels/${channelId}/forum/posts/${postId}/replies`),
  createReply: (channelId: string, postId: string, data: CreateForumReplyRequest) =>
    post<ForumReply>(`/channels/${channelId}/forum/posts/${postId}/replies`, data),
};

// ── Stickers ─────────────────────────────────────────────────
export const stickers = {
  list: (serverId: string) => get<Sticker[]>(`/servers/${serverId}/stickers`),
  create: (serverId: string, name: string, file: File) => {
    const fd = new FormData();
    fd.append('name', name);
    fd.append('file', file);
    return upload<Sticker>(`/servers/${serverId}/stickers`, fd);
  },
  delete: (serverId: string, stickerId: string) =>
    del<void>(`/servers/${serverId}/stickers/${stickerId}`),
};

// ── Badges ───────────────────────────────────────────────────
export const badges = {
  create: (data: { name: string; icon_url: string; description: string }) =>
    post<Badge>('/badges', data),
  award: (userId: string, badgeId: string) =>
    post<void>(`/users/${userId}/badges/${badgeId}`),
  revoke: (userId: string, badgeId: string) =>
    del<void>(`/users/${userId}/badges/${badgeId}`),
};

// ── Modération ───────────────────────────────────────────────
export const moderation = {
  kick: (serverId: string, userId: string, reason?: string) =>
    post<void>(`/servers/${serverId}/kick/${userId}`, reason ? { reason } : undefined),
  ban: (serverId: string, userId: string, reason?: string) =>
    post<void>(`/servers/${serverId}/ban/${userId}`, reason ? { reason } : undefined),
  unban: (serverId: string, userId: string) =>
    post<void>(`/servers/${serverId}/unban/${userId}`),
  mute: (serverId: string, userId: string, duration_minutes: number, reason?: string) =>
    post<void>(`/servers/${serverId}/mute/${userId}`, { duration_minutes, reason }),
  unmute: (serverId: string, userId: string) =>
    post<void>(`/servers/${serverId}/unmute/${userId}`),
  auditLog: (serverId: string) => get<AuditLogEntry[]>(`/servers/${serverId}/audit-log`),
};

// ── Upload générique ─────────────────────────────────────────
export const uploads = {
  upload: (file: File) => {
    const fd = new FormData();
    fd.append('file', file);
    return upload<{ url: string }>('/upload', fd);
  },
};
