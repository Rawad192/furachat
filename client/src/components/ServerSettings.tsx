// Paramètres du serveur — vue complète avec onglets
import { useState } from 'react';
import { useUiStore } from '../stores/uiStore';
import { useAuthStore } from '../stores/authStore';
import {
  servers as serversApi,
  roles as rolesApi,
  invitations as invitationsApi,
  channels as channelsApi,
} from '../services/api';
import type { Role, Invitation, CreateChannelRequest } from '../types';
import styles from './ServerSettings.module.css';

type Tab = 'general' | 'roles' | 'channels' | 'invitations' | 'moderation';

export default function ServerSettings() {
  const { serverDetails, setCurrentView, roles } = useUiStore();
  const user = useAuthStore((s) => s.user);
  const [tab, setTab] = useState<Tab>('general');

  if (!serverDetails) return null;

  const isOwner = serverDetails.server.owner_id === user?.id;

  return (
    <div className={styles.container}>
      <div className={styles.sidebar}>
        <div className={styles.sidebarTitle}>{serverDetails.server.name}</div>
        {(['general', 'roles', 'channels', 'invitations', 'moderation'] as Tab[]).map((t) => (
          <div
            key={t}
            className={`${styles.sidebarItem} ${tab === t ? styles.active : ''}`}
            onClick={() => setTab(t)}
          >
            {{ general: 'Général', roles: 'Rôles', channels: 'Canaux', invitations: 'Invitations', moderation: 'Modération' }[t]}
          </div>
        ))}
      </div>

      <div className={styles.content}>
        <div className={styles.back} onClick={() => setCurrentView('chat')}>
          \u2190 Retour
        </div>

        {tab === 'general' && <GeneralTab isOwner={isOwner} />}
        {tab === 'roles' && <RolesTab />}
        {tab === 'channels' && <ChannelsTab />}
        {tab === 'invitations' && <InvitationsTab />}
        {tab === 'moderation' && <ModerationTab />}
      </div>
    </div>
  );
}

// ── Onglet Général ───────────────────────────────────────────
function GeneralTab({ isOwner }: { isOwner: boolean }) {
  const { serverDetails, setServerDetails, setServerList, serverList, setCurrentServer, setCurrentView } = useUiStore();
  const [name, setName] = useState(serverDetails?.server.name || '');
  const [description, setDescription] = useState(serverDetails?.server.description || '');
  const [saving, setSaving] = useState(false);

  if (!serverDetails) return null;

  const handleSave = async () => {
    setSaving(true);
    try {
      const updated = await serversApi.update(serverDetails.server.id, {
        name: name.trim() || undefined,
        description: description.trim() || null,
      });
      setServerDetails({ ...serverDetails, server: updated });
      setServerList(serverList.map((s) => (s.id === updated.id ? updated : s)));
    } catch (e) {
      console.error('Erreur mise à jour serveur:', e);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!confirm('Supprimer définitivement ce serveur ? Cette action est irréversible.')) return;
    try {
      await serversApi.delete(serverDetails.server.id);
      setServerList(serverList.filter((s) => s.id !== serverDetails.server.id));
      setCurrentServer(null);
      setCurrentView('chat');
    } catch (e) {
      console.error('Erreur suppression serveur:', e);
    }
  };

  return (
    <div className={styles.section}>
      <h2 className={styles.sectionTitle}>Paramètres généraux</h2>
      <div className={styles.field}>
        <label className={styles.label}>Nom du serveur</label>
        <input className={styles.input} value={name} onChange={(e) => setName(e.target.value)} maxLength={100} />
      </div>
      <div className={styles.field}>
        <label className={styles.label}>Description</label>
        <textarea className={styles.textarea} value={description} onChange={(e) => setDescription(e.target.value)} maxLength={500} />
      </div>
      <button className={styles.btnPrimary} onClick={handleSave} disabled={saving}>
        {saving ? 'Sauvegarde...' : 'Sauvegarder'}
      </button>

      {isOwner && (
        <div className={styles.dangerZone}>
          <div className={styles.dangerTitle}>Zone dangereuse</div>
          <button className={styles.btnDanger} onClick={handleDelete}>
            Supprimer le serveur
          </button>
        </div>
      )}
    </div>
  );
}

// ── Onglet Rôles ─────────────────────────────────────────────
function RolesTab() {
  const { serverDetails, roles } = useUiStore();
  const [newName, setNewName] = useState('');
  const [newColor, setNewColor] = useState('#e94560');

  if (!serverDetails) return null;

  const handleCreate = async () => {
    if (!newName.trim()) return;
    try {
      const role = await rolesApi.create(serverDetails.server.id, {
        name: newName.trim(),
        color: newColor,
      });
      // Recharger les détails du serveur
      const details = await serversApi.get(serverDetails.server.id);
      useUiStore.getState().setServerDetails(details);
      setNewName('');
    } catch (e) {
      console.error('Erreur création rôle:', e);
    }
  };

  const handleDelete = async (roleId: string) => {
    try {
      await rolesApi.delete(serverDetails.server.id, roleId);
      const details = await serversApi.get(serverDetails.server.id);
      useUiStore.getState().setServerDetails(details);
    } catch (e) {
      console.error('Erreur suppression rôle:', e);
    }
  };

  return (
    <div className={styles.section}>
      <h2 className={styles.sectionTitle}>Rôles</h2>

      <div className={styles.roleList}>
        {roles.map((role) => (
          <div key={role.id} className={styles.roleItem}>
            <div className={styles.roleColor} style={{ background: role.color || '#808080' }} />
            <span className={styles.roleName}>{role.name}</span>
            {!role.is_default && (
              <div className={styles.roleActions}>
                <button className={styles.btnDanger} style={{ padding: '4px 10px', fontSize: 12 }} onClick={() => handleDelete(role.id)}>
                  Supprimer
                </button>
              </div>
            )}
          </div>
        ))}
      </div>

      <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12, color: 'var(--text-secondary)' }}>
        Nouveau rôle
      </h3>
      <div style={{ display: 'flex', gap: 12, alignItems: 'flex-end' }}>
        <div className={styles.field} style={{ flex: 1, marginBottom: 0 }}>
          <label className={styles.label}>Nom</label>
          <input className={styles.input} value={newName} onChange={(e) => setNewName(e.target.value)} placeholder="Modérateur" />
        </div>
        <div className={styles.field} style={{ marginBottom: 0 }}>
          <label className={styles.label}>Couleur</label>
          <input type="color" value={newColor} onChange={(e) => setNewColor(e.target.value)} style={{ width: 44, height: 38, border: 'none', background: 'none', cursor: 'pointer' }} />
        </div>
        <button className={styles.btnPrimary} onClick={handleCreate} style={{ height: 38 }}>Créer</button>
      </div>
    </div>
  );
}

// ── Onglet Canaux ────────────────────────────────────────────
function ChannelsTab() {
  const { serverDetails, channels, categories } = useUiStore();
  const [newChannelName, setNewChannelName] = useState('');
  const [newChannelType, setNewChannelType] = useState<'text' | 'voice' | 'forum' | 'announcement'>('text');
  const [newCategoryName, setNewCategoryName] = useState('');

  if (!serverDetails) return null;

  const handleCreateChannel = async () => {
    if (!newChannelName.trim()) return;
    try {
      await channelsApi.create(serverDetails.server.id, {
        name: newChannelName.trim(),
        channel_type: newChannelType,
      });
      const details = await serversApi.get(serverDetails.server.id);
      useUiStore.getState().setServerDetails(details);
      setNewChannelName('');
    } catch (e) {
      console.error('Erreur création canal:', e);
    }
  };

  const handleCreateCategory = async () => {
    if (!newCategoryName.trim()) return;
    try {
      await channelsApi.createCategory(serverDetails.server.id, { name: newCategoryName.trim() });
      const details = await serversApi.get(serverDetails.server.id);
      useUiStore.getState().setServerDetails(details);
      setNewCategoryName('');
    } catch (e) {
      console.error('Erreur création catégorie:', e);
    }
  };

  const handleDeleteChannel = async (channelId: string) => {
    try {
      await channelsApi.delete(serverDetails.server.id, channelId);
      const details = await serversApi.get(serverDetails.server.id);
      useUiStore.getState().setServerDetails(details);
    } catch (e) {
      console.error('Erreur suppression canal:', e);
    }
  };

  return (
    <div className={styles.section}>
      <h2 className={styles.sectionTitle}>Canaux</h2>

      <div className={styles.roleList}>
        {channels.map((ch) => (
          <div key={ch.id} className={styles.roleItem}>
            <span style={{ color: 'var(--text-muted)' }}>
              {{ text: '#', voice: '\u{1F50A}', forum: '\u{1F4CB}', announcement: '\u{1F4E2}' }[ch.channel_type]}
            </span>
            <span className={styles.roleName}>{ch.name}</span>
            <button className={styles.btnDanger} style={{ padding: '4px 10px', fontSize: 12 }} onClick={() => handleDeleteChannel(ch.id)}>
              Supprimer
            </button>
          </div>
        ))}
      </div>

      <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12, color: 'var(--text-secondary)' }}>
        Nouveau canal
      </h3>
      <div style={{ display: 'flex', gap: 12, alignItems: 'flex-end', marginBottom: 24 }}>
        <div className={styles.field} style={{ flex: 1, marginBottom: 0 }}>
          <label className={styles.label}>Nom</label>
          <input className={styles.input} value={newChannelName} onChange={(e) => setNewChannelName(e.target.value)} placeholder="discussion" />
        </div>
        <div className={styles.field} style={{ marginBottom: 0 }}>
          <label className={styles.label}>Type</label>
          <select value={newChannelType} onChange={(e) => setNewChannelType(e.target.value as typeof newChannelType)} style={{ padding: '10px 12px', background: 'var(--bg-input)', color: 'var(--text-primary)', border: '1px solid var(--border-color)', borderRadius: 'var(--radius-md)' }}>
            <option value="text">Texte</option>
            <option value="voice">Vocal</option>
            <option value="forum">Forum</option>
            <option value="announcement">Annonce</option>
          </select>
        </div>
        <button className={styles.btnPrimary} onClick={handleCreateChannel} style={{ height: 38 }}>Créer</button>
      </div>

      <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12, color: 'var(--text-secondary)' }}>
        Nouvelle catégorie
      </h3>
      <div style={{ display: 'flex', gap: 12, alignItems: 'flex-end' }}>
        <div className={styles.field} style={{ flex: 1, marginBottom: 0 }}>
          <label className={styles.label}>Nom</label>
          <input className={styles.input} value={newCategoryName} onChange={(e) => setNewCategoryName(e.target.value)} placeholder="Général" />
        </div>
        <button className={styles.btnPrimary} onClick={handleCreateCategory} style={{ height: 38 }}>Créer</button>
      </div>
    </div>
  );
}

// ── Onglet Invitations ───────────────────────────────────────
function InvitationsTab() {
  const { serverDetails } = useUiStore();
  const [invite, setInvite] = useState<Invitation | null>(null);
  const [creating, setCreating] = useState(false);

  if (!serverDetails) return null;

  const handleCreate = async () => {
    setCreating(true);
    try {
      const inv = await invitationsApi.create(serverDetails.server.id, { expires_in_hours: 24 });
      setInvite(inv);
    } catch (e) {
      console.error('Erreur création invitation:', e);
    } finally {
      setCreating(false);
    }
  };

  const handleCopy = () => {
    if (invite) {
      navigator.clipboard.writeText(invite.code);
    }
  };

  return (
    <div className={styles.section}>
      <h2 className={styles.sectionTitle}>Invitations</h2>

      {invite && (
        <div className={styles.inviteCode}>
          <span className={styles.inviteCodeText}>{invite.code}</span>
          <button className={styles.copyBtn} onClick={handleCopy}>Copier</button>
        </div>
      )}

      <button className={styles.btnPrimary} onClick={handleCreate} disabled={creating}>
        {creating ? 'Création...' : 'Générer une invitation'}
      </button>
      <p style={{ fontSize: 13, color: 'var(--text-muted)', marginTop: 8 }}>
        L'invitation expire après 24 heures.
      </p>
    </div>
  );
}

// ── Onglet Modération ────────────────────────────────────────
function ModerationTab() {
  const { serverDetails, members } = useUiStore();
  const [auditLog, setAuditLog] = useState<import('../types').AuditLogEntry[]>([]);
  const [loaded, setLoaded] = useState(false);
  const user = useAuthStore((s) => s.user);

  if (!serverDetails) return null;

  const loadAudit = async () => {
    try {
      const { moderation } = await import('../services/api');
      const log = await moderation.auditLog(serverDetails.server.id);
      setAuditLog(log);
      setLoaded(true);
    } catch (e) {
      console.error('Erreur chargement audit:', e);
    }
  };

  const handleKick = async (userId: string) => {
    const reason = prompt('Raison du kick (optionnel):');
    try {
      const { moderation } = await import('../services/api');
      await moderation.kick(serverDetails.server.id, userId, reason || undefined);
      // Recharger les membres
      const details = await serversApi.get(serverDetails.server.id);
      useUiStore.getState().setServerDetails(details);
    } catch (e) {
      console.error('Erreur kick:', e);
    }
  };

  const handleBan = async (userId: string) => {
    const reason = prompt('Raison du ban (optionnel):');
    try {
      const { moderation } = await import('../services/api');
      await moderation.ban(serverDetails.server.id, userId, reason || undefined);
      const details = await serversApi.get(serverDetails.server.id);
      useUiStore.getState().setServerDetails(details);
    } catch (e) {
      console.error('Erreur ban:', e);
    }
  };

  return (
    <div className={styles.section}>
      <h2 className={styles.sectionTitle}>Modération</h2>

      <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12, color: 'var(--text-secondary)' }}>
        Membres ({members.length})
      </h3>
      <div className={styles.roleList}>
        {members
          .filter((m) => m.user_id !== user?.id)
          .map((m) => (
            <div key={m.user_id} className={styles.roleItem}>
              <span className={styles.roleName}>{m.display_name || m.username}</span>
              <div className={styles.roleActions}>
                <button className={styles.btnSecondary} style={{ padding: '4px 10px', fontSize: 12 }} onClick={() => handleKick(m.user_id)}>
                  Kick
                </button>
                <button className={styles.btnDanger} style={{ padding: '4px 10px', fontSize: 12 }} onClick={() => handleBan(m.user_id)}>
                  Ban
                </button>
              </div>
            </div>
          ))}
      </div>

      <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12, marginTop: 24, color: 'var(--text-secondary)' }}>
        Journal d'audit
      </h3>
      {!loaded ? (
        <button className={styles.btnSecondary} onClick={loadAudit}>Charger le journal</button>
      ) : (
        <div className={styles.roleList}>
          {auditLog.length === 0 && (
            <p style={{ color: 'var(--text-muted)', fontSize: 13 }}>Aucune entrée.</p>
          )}
          {auditLog.map((entry) => (
            <div key={entry.id} className={styles.roleItem} style={{ flexDirection: 'column', alignItems: 'flex-start', gap: 4 }}>
              <div style={{ fontSize: 13 }}>
                <strong>{entry.moderator_username}</strong> — {entry.action}
              </div>
              {entry.reason && <div style={{ fontSize: 12, color: 'var(--text-muted)' }}>Raison : {entry.reason}</div>}
              <div style={{ fontSize: 11, color: 'var(--text-muted)' }}>
                {new Date(entry.created_at).toLocaleString('fr-FR')}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
