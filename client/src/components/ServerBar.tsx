// Barre de serveurs horizontale (en haut) — icônes des serveurs + bouton créer/rejoindre
import { useState } from 'react';
import { useUiStore } from '../stores/uiStore';
import { servers as serversApi, invitations as invitationsApi } from '../services/api';
import styles from './ServerBar.module.css';

export default function ServerBar() {
  const { serverList, currentServerId, setCurrentServer, setCurrentChannel, setServerList } = useUiStore();
  const [showModal, setShowModal] = useState(false);

  const handleSelectServer = (id: string) => {
    if (id === currentServerId) return;
    setCurrentChannel(null);
    setCurrentServer(id);
  };

  const handleHome = () => {
    setCurrentServer(null);
    setCurrentChannel(null);
  };

  return (
    <div className={styles.bar}>
      {/* Bouton accueil */}
      <div className={styles.homeButton} onClick={handleHome} title="Accueil">
        F
      </div>

      <div className={styles.separator} />

      {/* Liste des serveurs */}
      {serverList.map((server) => (
        <div
          key={server.id}
          className={`${styles.serverIcon} ${server.id === currentServerId ? styles.active : ''}`}
          onClick={() => handleSelectServer(server.id)}
          title={server.name}
        >
          {server.icon_url ? (
            <img src={server.icon_url} alt={server.name} />
          ) : (
            server.name.charAt(0).toUpperCase()
          )}
        </div>
      ))}

      {/* Bouton ajouter */}
      <div className={styles.addButton} onClick={() => setShowModal(true)} title="Créer ou rejoindre un serveur">
        +
      </div>

      {showModal && (
        <ServerModal
          onClose={() => setShowModal(false)}
          onCreated={(server) => {
            setServerList([...serverList, server]);
            setCurrentServer(server.id);
            setCurrentChannel(null);
            setShowModal(false);
          }}
        />
      )}
    </div>
  );
}

// Modale créer / rejoindre un serveur
function ServerModal({
  onClose,
  onCreated,
}: {
  onClose: () => void;
  onCreated: (server: import('../types').Server) => void;
}) {
  const [tab, setTab] = useState<'create' | 'join'>('create');
  const [name, setName] = useState('');
  const [code, setCode] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleCreate = async () => {
    if (!name.trim()) return;
    setLoading(true);
    setError('');
    try {
      const server = await serversApi.create({ name: name.trim() });
      onCreated(server);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  };

  const handleJoin = async () => {
    if (!code.trim()) return;
    setLoading(true);
    setError('');
    try {
      const server = await invitationsApi.join(code.trim());
      onCreated(server);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.modalTitle}>Ajouter un serveur</div>

        <div className={styles.tabs}>
          <div
            className={`${styles.tab} ${tab === 'create' ? styles.activeTab : ''}`}
            onClick={() => setTab('create')}
          >
            Créer
          </div>
          <div
            className={`${styles.tab} ${tab === 'join' ? styles.activeTab : ''}`}
            onClick={() => setTab('join')}
          >
            Rejoindre
          </div>
        </div>

        {error && <div style={{ color: 'var(--accent-danger)', fontSize: 13, marginBottom: 12 }}>{error}</div>}

        {tab === 'create' ? (
          <>
            <div className={styles.modalField}>
              <label className={styles.modalLabel}>Nom du serveur</label>
              <input
                className={styles.modalInput}
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Mon serveur"
                maxLength={100}
              />
            </div>
            <div className={styles.modalActions}>
              <button className={styles.modalBtnSecondary} onClick={onClose}>Annuler</button>
              <button className={styles.modalBtnPrimary} onClick={handleCreate} disabled={loading}>
                {loading ? '...' : 'Créer'}
              </button>
            </div>
          </>
        ) : (
          <>
            <div className={styles.modalField}>
              <label className={styles.modalLabel}>Code d'invitation</label>
              <input
                className={styles.modalInput}
                value={code}
                onChange={(e) => setCode(e.target.value)}
                placeholder="AbCdEfGh"
                maxLength={20}
              />
            </div>
            <div className={styles.modalActions}>
              <button className={styles.modalBtnSecondary} onClick={onClose}>Annuler</button>
              <button className={styles.modalBtnPrimary} onClick={handleJoin} disabled={loading}>
                {loading ? '...' : 'Rejoindre'}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
