// Vue Vocal — salon vocal avec contrôles audio et participants
import { useState, useEffect } from 'react';
import { useUiStore } from '../stores/uiStore';
import { useAuthStore } from '../stores/authStore';
import { wsClient } from '../services/wsClient';
import { setRNNoiseEnabled, isRNNoiseEnabled } from '../services/rnnoise';
import styles from './VoiceView.module.css';

interface VoiceParticipant {
  user_id: string;
  username: string;
  avatar_url: string | null;
  muted: boolean;
  deafened: boolean;
}

export default function VoiceView() {
  const { currentChannelId, channels } = useUiStore();
  const user = useAuthStore((s) => s.user);
  const channel = channels.find((c) => c.id === currentChannelId);

  const [joined, setJoined] = useState(false);
  const [muted, setMuted] = useState(false);
  const [deafened, setDeafened] = useState(false);
  const [noiseSuppress, setNoiseSuppress] = useState(isRNNoiseEnabled());
  const [participants, setParticipants] = useState<VoiceParticipant[]>([]);

  // Écouter les événements vocal
  useEffect(() => {
    const unsubs: (() => void)[] = [];

    unsubs.push(wsClient.on('VOICE_USER_JOINED', (data) => {
      const d = data as Record<string, unknown>;
      if (d.channel_id !== currentChannelId) return;
      setParticipants((prev) => {
        if (prev.some((p) => p.user_id === d.user_id)) return prev;
        return [...prev, {
          user_id: d.user_id as string,
          username: d.username as string,
          avatar_url: (d.avatar_url as string) || null,
          muted: false,
          deafened: false,
        }];
      });
    }));

    unsubs.push(wsClient.on('VOICE_USER_LEFT', (data) => {
      const d = data as Record<string, unknown>;
      setParticipants((prev) => prev.filter((p) => p.user_id !== d.user_id));
    }));

    return () => unsubs.forEach((fn) => fn());
  }, [currentChannelId]);

  const handleJoin = () => {
    if (!currentChannelId) return;
    wsClient.send({ type: 'VOICE_JOIN', channel_id: currentChannelId });
    setJoined(true);
    // S'ajouter soi-même aux participants
    if (user) {
      setParticipants((prev) => [
        ...prev,
        { user_id: user.id, username: user.username, avatar_url: user.avatar_url, muted: false, deafened: false },
      ]);
    }
  };

  const handleLeave = () => {
    if (!currentChannelId) return;
    wsClient.send({ type: 'VOICE_LEAVE', channel_id: currentChannelId });
    setJoined(false);
    setParticipants([]);
  };

  const toggleMute = () => {
    setMuted(!muted);
  };

  const toggleDeafen = () => {
    setDeafened(!deafened);
    if (!deafened) setMuted(true);
  };

  const toggleNoise = () => {
    const next = !noiseSuppress;
    setNoiseSuppress(next);
    setRNNoiseEnabled(next);
  };

  if (!currentChannelId || !channel) {
    return (
      <div className={styles.container}>
        <div style={{ color: 'var(--text-muted)', fontSize: 16 }}>
          Sélectionnez un salon vocal
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.channelName}>{'\u{1F50A}'} {channel.name}</div>
        <div className={styles.channelSub}>
          {joined
            ? `Connecté — ${participants.length} participant${participants.length !== 1 ? 's' : ''}`
            : 'Non connecté'}
        </div>
      </div>

      {/* Participants */}
      {participants.length > 0 && (
        <div className={styles.participants}>
          {participants.map((p) => (
            <div key={p.user_id} className={styles.participant}>
              <div className={styles.participantAvatar}>
                {p.avatar_url ? (
                  <img src={p.avatar_url} alt={p.username} />
                ) : (
                  p.username.charAt(0).toUpperCase()
                )}
              </div>
              <div className={styles.participantName}>{p.username}</div>
              {p.muted && <div className={styles.participantMuted}>Muet</div>}
            </div>
          ))}
        </div>
      )}

      {/* Contrôles */}
      {joined ? (
        <>
          <div className={styles.controls}>
            <button
              className={`${styles.muteBtn} ${muted ? styles.muted : ''}`}
              onClick={toggleMute}
              title={muted ? 'Activer le micro' : 'Couper le micro'}
            >
              {muted ? '\u{1F507}' : '\u{1F3A4}'}
            </button>
            <button
              className={`${styles.deafenBtn} ${deafened ? styles.deafened : ''}`}
              onClick={toggleDeafen}
              title={deafened ? 'Activer le son' : 'Couper le son'}
            >
              {deafened ? '\u{1F515}' : '\u{1F50A}'}
            </button>
            <button className={styles.leaveBtn} onClick={handleLeave} title="Quitter">
              {'\u{260E}'}
            </button>
          </div>

          <label className={styles.noiseToggle}>
            <input type="checkbox" checked={noiseSuppress} onChange={toggleNoise} />
            Suppression de bruit (RNNoise)
          </label>
        </>
      ) : (
        <button className={styles.joinBtn} onClick={handleJoin}>
          Rejoindre le salon vocal
        </button>
      )}
    </div>
  );
}
