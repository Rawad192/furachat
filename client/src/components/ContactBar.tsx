// Barre de contacts — liste horizontale des amis en bas de l'écran
import { useUiStore } from '../stores/uiStore';
import type { Friend } from '../types';
import styles from './ContactBar.module.css';

export default function ContactBar() {
  const { friends } = useUiStore();

  const statusClass = (status: string) => {
    switch (status) {
      case 'online': return styles.online;
      case 'idle': return styles.idle;
      case 'dnd': return styles.dnd;
      default: return styles.offline;
    }
  };

  // Trier : en ligne d'abord
  const sorted = [...friends].sort((a, b) => {
    const order = ['online', 'idle', 'dnd', 'offline'];
    return order.indexOf(a.status) - order.indexOf(b.status);
  });

  return (
    <div className={styles.bar}>
      <span className={styles.label}>Amis</span>
      {sorted.length === 0 && <span className={styles.empty}>Aucun ami ajouté</span>}
      {sorted.map((friend) => (
        <ContactItem key={friend.user_id} friend={friend} statusClass={statusClass} />
      ))}
    </div>
  );
}

function ContactItem({
  friend,
  statusClass,
}: {
  friend: Friend;
  statusClass: (s: string) => string;
}) {
  return (
    <div className={styles.contact} title={friend.display_name || friend.username}>
      <div className={styles.avatar}>
        {friend.avatar_url ? (
          <img src={friend.avatar_url} alt={friend.username} />
        ) : (
          friend.username.charAt(0).toUpperCase()
        )}
        <div className={`${styles.statusDot} ${statusClass(friend.status)}`} />
      </div>
      <span className={styles.name}>{friend.display_name || friend.username}</span>
    </div>
  );
}
