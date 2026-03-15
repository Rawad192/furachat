// Liste des membres du serveur — panneau droit
import { useUiStore } from '../stores/uiStore';
import type { ServerMember } from '../types';
import styles from './MemberList.module.css';

export default function MemberList() {
  const { members, roles } = useUiStore();

  // Trier les rôles par position (plus haute = premier affiché)
  const sortedRoles = [...roles].filter((r) => !r.is_default).sort((a, b) => b.position - a.position);

  // Séparer les membres online/offline
  const online = members.filter((m) => m.status !== 'offline');
  const offline = members.filter((m) => m.status === 'offline');

  // Grouper les membres online par leur rôle le plus haut
  const grouped = new Map<string, ServerMember[]>();
  const noRole: ServerMember[] = [];

  for (const member of online) {
    if (member.roles.length === 0) {
      noRole.push(member);
      continue;
    }
    // Trouver le rôle le plus haut (position la plus élevée)
    const topRole = member.roles.reduce((best, r) => (r.position > best.position ? r : best), member.roles[0]);
    if (topRole.is_default) {
      noRole.push(member);
      continue;
    }
    const list = grouped.get(topRole.id) || [];
    list.push(member);
    grouped.set(topRole.id, list);
  }

  const statusClass = (status: string) => {
    switch (status) {
      case 'online': return styles.statusOnline;
      case 'idle': return styles.statusIdle;
      case 'dnd': return styles.statusDnd;
      default: return styles.statusOffline;
    }
  };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>Membres — {members.length}</div>
      <div className={styles.list}>
        {/* Groupes par rôle */}
        {sortedRoles.map((role) => {
          const roleMembers = grouped.get(role.id);
          if (!roleMembers || roleMembers.length === 0) return null;
          return (
            <div key={role.id}>
              <div className={styles.roleGroup} style={{ color: role.color || undefined }}>
                {role.name} — {roleMembers.length}
              </div>
              {roleMembers.map((m) => (
                <MemberItem key={m.user_id} member={m} statusClass={statusClass} />
              ))}
            </div>
          );
        })}

        {/* Membres sans rôle spécial (en ligne) */}
        {noRole.length > 0 && (
          <>
            <div className={styles.roleGroup}>En ligne — {noRole.length}</div>
            {noRole.map((m) => (
              <MemberItem key={m.user_id} member={m} statusClass={statusClass} />
            ))}
          </>
        )}

        {/* Hors ligne */}
        {offline.length > 0 && (
          <>
            <div className={styles.roleGroup}>Hors ligne — {offline.length}</div>
            {offline.map((m) => (
              <MemberItem key={m.user_id} member={m} statusClass={statusClass} />
            ))}
          </>
        )}
      </div>
    </div>
  );
}

function MemberItem({
  member,
  statusClass,
}: {
  member: ServerMember;
  statusClass: (s: string) => string;
}) {
  // Couleur du nom = couleur du rôle le plus haut
  const topColor = member.roles.length > 0
    ? member.roles.reduce((best, r) => (r.position > best.position ? r : best), member.roles[0]).color
    : undefined;

  return (
    <div className={styles.member}>
      <div className={styles.avatar}>
        {member.avatar_url ? (
          <img src={member.avatar_url} alt={member.username} />
        ) : (
          member.username.charAt(0).toUpperCase()
        )}
        <div className={`${styles.statusDot} ${statusClass(member.status)}`} />
      </div>
      <div className={styles.info}>
        <div className={styles.name} style={{ color: topColor || undefined }}>
          {member.display_name || member.username}
        </div>
        {member.custom_status && (
          <div className={styles.customStatus}>{member.custom_status}</div>
        )}
      </div>
    </div>
  );
}
