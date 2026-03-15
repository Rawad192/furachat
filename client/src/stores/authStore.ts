// Store d'authentification — gère token, utilisateur, login/register/logout
import { create } from 'zustand';
import type { User } from '../types';
import { auth, users, ApiError } from '../services/api';
import { wsClient } from '../services/wsClient';

interface AuthState {
  user: User | null;
  token: string | null;
  loading: boolean;
  error: string | null;

  login: (email: string, password: string) => Promise<void>;
  register: (username: string, email: string, password: string) => Promise<void>;
  logout: () => void;
  restoreSession: () => Promise<void>;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  token: localStorage.getItem('token'),
  loading: false,
  error: null,

  login: async (email, password) => {
    set({ loading: true, error: null });
    try {
      const res = await auth.login({ email, password });
      localStorage.setItem('token', res.token);
      wsClient.connect(res.token);
      set({ user: res.user, token: res.token, loading: false });
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : 'Erreur de connexion';
      set({ loading: false, error: msg });
    }
  },

  register: async (username, email, password) => {
    set({ loading: true, error: null });
    try {
      const res = await auth.register({ username, email, password });
      localStorage.setItem('token', res.token);
      wsClient.connect(res.token);
      set({ user: res.user, token: res.token, loading: false });
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : "Erreur d'inscription";
      set({ loading: false, error: msg });
    }
  },

  logout: () => {
    localStorage.removeItem('token');
    wsClient.disconnect();
    set({ user: null, token: null });
  },

  restoreSession: async () => {
    const token = localStorage.getItem('token');
    if (!token) return;
    set({ loading: true });
    try {
      const user = await users.me();
      wsClient.connect(token);
      set({ user, token, loading: false });
    } catch {
      // Token invalide ou expiré
      localStorage.removeItem('token');
      set({ user: null, token: null, loading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
