// Layout principal — orchestre tous les panneaux de l'application
import { useEffect } from 'react';
import { useUiStore } from '../stores/uiStore';
import { useAuthStore } from '../stores/authStore';
import { servers as serversApi, friends as friendsApi } from '../services/api';
import { wsClient } from '../services/wsClient';
import ServerBar from './ServerBar';
import ChannelSidebar from './ChannelSidebar';
import ChatArea from './ChatArea';
import ForumView from './ForumView';
import VoiceView from './VoiceView';
import ServerSettings from './ServerSettings';
import MemberList from './MemberList';
import ContactBar from './ContactBar';
import styles from './MainLayout.module.css';

export default function MainLayout() {
  const { currentServerId, currentView, setServerList, setServerDetails, setFriends, memberListVisible, contactBarVisible } = useUiStore();
  const user = useAuthStore((s) => s.user);

  // Charger la liste des serveurs et amis au montage
  useEffect(() => {
    serversApi.list().then((list) => useUiStore.getState().setServerList(list)).catch(console.error);
    friendsApi.list().then((list) => useUiStore.getState().setFriends(list)).catch(console.error);
  }, []);

  // Charger les détails du serveur sélectionné
  useEffect(() => {
    if (currentServerId) {
      serversApi.get(currentServerId).then((details) => {
        setServerDetails(details);
        // Sélectionner le premier canal texte par défaut
        const firstText = details.channels.find((c) => c.channel_type === 'text');
        if (firstText && !useUiStore.getState().currentChannelId) {
          useUiStore.getState().setCurrentChannel(firstText.id);
        }
      }).catch(console.error);
    } else {
      setServerDetails(null);
    }
  }, [currentServerId, setServerDetails]);

  // Écouter les événements WebSocket
  useEffect(() => {
    const unsubs: (() => void)[] = [];

    unsubs.push(wsClient.on('MESSAGE_NEW', (data) => {
      const msg = data as Record<string, unknown>;
      if (msg.channel_id === useUiStore.getState().currentChannelId) {
        useUiStore.getState().addMessage(msg as never);
      }
    }));

    unsubs.push(wsClient.on('MESSAGE_UPDATED', (data) => {
      const msg = data as Record<string, unknown>;
      useUiStore.getState().updateMessage(
        msg.message_id as string,
        msg.content as string,
        msg.edited_at as string
      );
    }));

    unsubs.push(wsClient.on('MESSAGE_DELETED', (data) => {
      const msg = data as Record<string, unknown>;
      useUiStore.getState().removeMessage(msg.message_id as string);
    }));

    unsubs.push(wsClient.on('FORUM_POST_NEW', (data) => {
      const post = data as Record<string, unknown>;
      if (post.channel_id === useUiStore.getState().currentChannelId) {
        useUiStore.getState().addForumPost(post as never);
      }
    }));

    return () => unsubs.forEach((fn) => fn());
  }, []);

  return (
    <div className={styles.layout}>
      <ServerBar />
      <div className={styles.middle}>
        {currentServerId && <ChannelSidebar />}
        {currentView === 'server-settings' ? (
          <ServerSettings />
        ) : currentView === 'forum' ? (
          <ForumView />
        ) : currentView === 'voice' ? (
          <VoiceView />
        ) : (
          <ChatArea />
        )}
        {currentServerId && memberListVisible && currentView === 'chat' && <MemberList />}
      </div>
      {contactBarVisible && <ContactBar />}
    </div>
  );
}
