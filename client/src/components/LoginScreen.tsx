// Écran de connexion et d'inscription
import React, { useState } from 'react';
import { useAuth } from '../hooks/useAuth';
import styles from './LoginScreen.module.css';

export default function LoginScreen() {
  const { login, register, loading, error, clearError } = useAuth();
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [username, setUsername] = useState('');

  const switchMode = () => {
    setMode(mode === 'login' ? 'register' : 'login');
    clearError();
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (mode === 'login') {
      await login(email, password);
    } else {
      await register(username, email, password);
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.card}>
        <div className={styles.logo}>
          <div className={styles.logoTitle}>FuraChat</div>
          <div className={styles.logoSub}>Messagerie auto-hébergée</div>
        </div>

        <h2 className={styles.title}>
          {mode === 'login' ? 'Connexion' : 'Créer un compte'}
        </h2>

        {error && <div className={styles.error}>{error}</div>}

        <form className={styles.form} onSubmit={handleSubmit}>
          {mode === 'register' && (
            <div className={styles.field}>
              <label className={styles.label} htmlFor="username">
                Nom d'utilisateur
              </label>
              <input
                id="username"
                className={styles.input}
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="MonPseudo"
                required
                minLength={2}
                maxLength={32}
                autoComplete="username"
              />
            </div>
          )}

          <div className={styles.field}>
            <label className={styles.label} htmlFor="email">
              Adresse e-mail
            </label>
            <input
              id="email"
              className={styles.input}
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="vous@exemple.com"
              required
              autoComplete="email"
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="password">
              Mot de passe
            </label>
            <input
              id="password"
              className={styles.input}
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="••••••••"
              required
              minLength={6}
              autoComplete={mode === 'login' ? 'current-password' : 'new-password'}
            />
          </div>

          <button className={styles.button} type="submit" disabled={loading}>
            {loading
              ? 'Chargement...'
              : mode === 'login'
                ? 'Se connecter'
                : "S'inscrire"}
          </button>
        </form>

        <div className={styles.switch}>
          {mode === 'login' ? (
            <>
              Pas encore de compte ?{' '}
              <span className={styles.switchLink} onClick={switchMode}>
                Créer un compte
              </span>
            </>
          ) : (
            <>
              Déjà un compte ?{' '}
              <span className={styles.switchLink} onClick={switchMode}>
                Se connecter
              </span>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
