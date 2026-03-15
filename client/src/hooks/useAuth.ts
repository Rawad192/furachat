// Hook d'authentification — restauration de session au montage
import { useEffect } from 'react';
import { useAuthStore } from '../stores/authStore';

export function useAuth() {
  const { user, token, loading, error, restoreSession, login, register, logout, clearError } =
    useAuthStore();

  useEffect(() => {
    if (token && !user) {
      restoreSession();
    }
  }, [token, user, restoreSession]);

  return { user, token, loading, error, login, register, logout, clearError };
}
