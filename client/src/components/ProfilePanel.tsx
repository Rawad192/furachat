// Panneau de profil — modification des infos utilisateur
import { useState, useRef } from 'react';
import { useAuthStore } from '../stores/authStore';
import { users } from '../services/api';
import styles from './ProfilePanel.module.css';

interface Props {
  onClose: () => void;
}

export default function ProfilePanel({ onClose }: Props) {
  const { user } = useAuthStore();
  const [displayName, setDisplayName] = useState(user?.display_name || '');
  const [bio, setBio] = useState(user?.bio || '');
  const [customStatus, setCustomStatus] = useState(user?.custom_status || '');
  const [saving, setSaving] = useState(false);
  const avatarRef = useRef<HTMLInputElement>(null);
  const bannerRef = useRef<HTMLInputElement>(null);

  if (!user) return null;

  const handleSave = async () => {
    setSaving(true);
    try {
      const updated = await users.update({
        display_name: displayName || null,
        bio: bio || null,
        custom_status: customStatus || null,
      });
      useAuthStore.setState({ user: updated });
      onClose();
    } catch (e) {
      console.error('Erreur mise à jour profil:', e);
    } finally {
      setSaving(false);
    }
  };

  const handleAvatarUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    try {
      const { url } = await users.uploadAvatar(file);
      useAuthStore.setState({ user: { ...user, avatar_url: url } });
    } catch (err) {
      console.error('Erreur upload avatar:', err);
    }
  };

  const handleBannerUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    try {
      const { url } = await users.uploadBanner(file);
      useAuthStore.setState({ user: { ...user, banner_url: url } });
    } catch (err) {
      console.error('Erreur upload bannière:', err);
    }
  };

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.panel} onClick={(e) => e.stopPropagation()}>
        {/* Bannière */}
        <div className={styles.banner}>
          {user.banner_url && <img src={user.banner_url} alt="bannière" />}
          <div className={styles.bannerUpload} onClick={() => bannerRef.current?.click()}>
            Changer
          </div>
          <input ref={bannerRef} type="file" accept="image/*" hidden onChange={handleBannerUpload} />
        </div>

        {/* Avatar + nom */}
        <div className={styles.avatarSection}>
          <div className={styles.avatar} onClick={() => avatarRef.current?.click()}>
            {user.avatar_url ? (
              <img src={user.avatar_url} alt={user.username} />
            ) : (
              user.username.charAt(0).toUpperCase()
            )}
            <input ref={avatarRef} type="file" accept="image/*" hidden onChange={handleAvatarUpload} />
          </div>
          <div className={styles.username}>{user.username}</div>
        </div>

        {/* Champs de modification */}
        <div className={styles.body}>
          <div className={styles.field}>
            <label className={styles.label}>Nom d'affichage</label>
            <input
              className={styles.input}
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder={user.username}
              maxLength={32}
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label}>Statut personnalisé</label>
            <input
              className={styles.input}
              value={customStatus}
              onChange={(e) => setCustomStatus(e.target.value)}
              placeholder="Que faites-vous ?"
              maxLength={128}
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label}>Bio</label>
            <textarea
              className={styles.textarea}
              value={bio}
              onChange={(e) => setBio(e.target.value)}
              placeholder="Parlez de vous..."
              maxLength={500}
            />
          </div>
        </div>

        {/* Actions */}
        <div className={styles.actions}>
          <button className={styles.btnSecondary} onClick={onClose}>Annuler</button>
          <button className={styles.btnPrimary} onClick={handleSave} disabled={saving}>
            {saving ? 'Sauvegarde...' : 'Sauvegarder'}
          </button>
        </div>
      </div>
    </div>
  );
}
