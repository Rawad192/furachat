// Sidebar des canaux — panneau gauche avec catégories, canaux et panneau utilisateur
import { useState } from 'react';
import { useUiStore } from '../stores/uiStore';
import { useAuthStore } from '../stores/authStore';
import type { Channel, Category } from '../types';
import styles from './ChannelSidebar.module.css';

// Icônes des types de canaux
const channelIcons: Record<string, string> = {
  text: '#',
  voice: '\u{1F50A}',
  forum: '\u{1F4CB}',
  announcement: '\u{1F4E2}',
};

export default function ChannelSidebar() {
  const { channels, categories, currentChannelId, setCurrentChannel, setCurrentView, serverDetails } = useUiStore();
  const { user, logout } = useAuthStore();
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set());

  const toggleCategory = (id: string) => {
    setCollapsedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleChannelClick = (channel: Channel) => {
    setCurrentChannel(channel.id);
    if (channel.channel_type === 'forum') {
      setCurrentView('forum');
    } else if (channel.channel_type === 'voice') {
      setCurrentView('voice');
    } else {
      setCurrentView('chat');
    }
  };

  // Grouper les canaux par catégorie
  const uncategorized = channels.filter((c) => !c.category_id);
  const sortedCategories = [...categories].sort((a, b) => a.position - b.position);

  const channelsByCategory = (catId: string) =>
    channels.filter((c) => c.category_id === catId).sort((a, b) => a.position - b.position);

  return (
    <div className={styles.sidebar}>
      {/* En-tête du serveur */}
      <div className={styles.header}>
        <span className={styles.headerName}>{serverDetails?.server.name ?? 'Serveur'}</span>
        <button
          className={styles.settingsBtn}
          onClick={() => setCurrentView('server-settings')}
          title="Paramètres du serveur"
        >
          \u2699
        </button>
      </div>

      {/* Liste des canaux */}
      <div className={styles.list}>
        {/* Canaux sans catégorie */}
        {uncategorized.map((ch) => (
          <ChannelItem
            key={ch.id}
            channel={ch}
            active={ch.id === currentChannelId}
            onClick={() => handleChannelClick(ch)}
          />
        ))}

        {/* Catégories */}
        {sortedCategories.map((cat) => {
          const collapsed = collapsedCategories.has(cat.id);
          const catChannels = channelsByCategory(cat.id);
          return (
            <div key={cat.id}>
              <div className={styles.category} onClick={() => toggleCategory(cat.id)}>
                <span>{cat.name}</span>
                <span className={`${styles.categoryArrow} ${collapsed ? styles.collapsed : ''}`}>
                  \u25BC
                </span>
              </div>
              {!collapsed &&
                catChannels.map((ch) => (
                  <ChannelItem
                    key={ch.id}
                    channel={ch}
                    active={ch.id === currentChannelId}
                    onClick={() => handleChannelClick(ch)}
                  />
                ))}
            </div>
          );
        })}
      </div>

      {/* Panneau utilisateur */}
      <div className={styles.userPanel}>
        <div className={styles.userAvatar}>
          {user?.avatar_url ? (
            <img src={user.avatar_url} alt={user.username} />
          ) : (
            user?.username.charAt(0).toUpperCase()
          )}
        </div>
        <div className={styles.userInfo}>
          <div className={styles.userName}>{user?.display_name || user?.username}</div>
          <div className={styles.userStatus}>{user?.custom_status || user?.status || 'En ligne'}</div>
        </div>
        <button className={styles.logoutBtn} onClick={logout} title="Déconnexion">
          \u23FB
        </button>
      </div>
    </div>
  );
}

function ChannelItem({
  channel,
  active,
  onClick,
}: {
  channel: Channel;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <div className={`${styles.channel} ${active ? styles.active : ''}`} onClick={onClick}>
      <span className={styles.channelIcon}>{channelIcons[channel.channel_type] || '#'}</span>
      <span className={styles.channelName}>{channel.name}</span>
    </div>
  );
}
