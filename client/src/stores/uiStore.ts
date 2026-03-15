// Store UI — gère l'état de navigation, serveur/canal sélectionné, panneaux
import { create } from 'zustand';
import type {
  Server,
  ServerDetails,
  Channel,
  Category,
  ServerMember,
  Role,
  Friend,
  Message,
  ForumPost,
} from '../types';

type View = 'chat' | 'forum' | 'voice' | 'server-settings' | 'profile';

interface UiState {
  // Navigation
  currentServerId: string | null;
  currentChannelId: string | null;
  currentView: View;

  // Données du serveur actif
  serverList: Server[];
  serverDetails: ServerDetails | null;
  channels: Channel[];
  categories: Category[];
  members: ServerMember[];
  roles: Role[];

  // Messages du canal actif
  messages: Message[];
  messagesLoading: boolean;
  hasMoreMessages: boolean;

  // Forum
  forumPosts: ForumPost[];

  // Amis (barre du bas)
  friends: Friend[];

  // Panneaux
  memberListVisible: boolean;
  contactBarVisible: boolean;

  // Actions
  setCurrentServer: (serverId: string | null) => void;
  setCurrentChannel: (channelId: string | null) => void;
  setCurrentView: (view: View) => void;
  setServerList: (servers: Server[]) => void;
  setServerDetails: (details: ServerDetails | null) => void;
  setMessages: (messages: Message[]) => void;
  prependMessages: (messages: Message[]) => void;
  addMessage: (message: Message) => void;
  updateMessage: (id: string, content: string, editedAt: string) => void;
  removeMessage: (id: string) => void;
  setMessagesLoading: (loading: boolean) => void;
  setHasMoreMessages: (has: boolean) => void;
  setForumPosts: (posts: ForumPost[]) => void;
  addForumPost: (post: ForumPost) => void;
  setFriends: (friends: Friend[]) => void;
  toggleMemberList: () => void;
  toggleContactBar: () => void;
}

export const useUiStore = create<UiState>((set) => ({
  currentServerId: null,
  currentChannelId: null,
  currentView: 'chat',

  serverList: [],
  serverDetails: null,
  channels: [],
  categories: [],
  members: [],
  roles: [],

  messages: [],
  messagesLoading: false,
  hasMoreMessages: true,

  forumPosts: [],

  friends: [],

  memberListVisible: true,
  contactBarVisible: true,

  setCurrentServer: (serverId) => set({ currentServerId: serverId }),
  setCurrentChannel: (channelId) => set({ currentChannelId: channelId, messages: [], hasMoreMessages: true }),
  setCurrentView: (view) => set({ currentView: view }),
  setServerList: (servers) => set({ serverList: servers }),
  setServerDetails: (details) =>
    set(details ? {
      serverDetails: details,
      channels: details.channels,
      categories: details.categories,
      members: details.members,
      roles: details.roles,
    } : {
      serverDetails: null,
      channels: [],
      categories: [],
      members: [],
      roles: [],
    }),
  setMessages: (messages) => set({ messages }),
  prependMessages: (older) => set((s) => ({ messages: [...older, ...s.messages] })),
  addMessage: (message) => set((s) => ({ messages: [...s.messages, message] })),
  updateMessage: (id, content, editedAt) =>
    set((s) => ({
      messages: s.messages.map((m) =>
        m.id === id ? { ...m, content, edited_at: editedAt } : m
      ),
    })),
  removeMessage: (id) =>
    set((s) => ({ messages: s.messages.filter((m) => m.id !== id) })),
  setMessagesLoading: (loading) => set({ messagesLoading: loading }),
  setHasMoreMessages: (has) => set({ hasMoreMessages: has }),
  setForumPosts: (posts) => set({ forumPosts: posts }),
  addForumPost: (post) => set((s) => ({ forumPosts: [post, ...s.forumPosts] })),
  setFriends: (friends) => set({ friends }),
  toggleMemberList: () => set((s) => ({ memberListVisible: !s.memberListVisible })),
  toggleContactBar: () => set((s) => ({ contactBarVisible: !s.contactBarVisible })),
}));
