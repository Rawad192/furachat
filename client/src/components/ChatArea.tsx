// Zone de chat principale — messages, saisie, réactions
import { useEffect, useRef, useState, useCallback } from 'react';
import { useUiStore } from '../stores/uiStore';
import { useAuthStore } from '../stores/authStore';
import { messages as messagesApi } from '../services/api';
import { wsClient } from '../services/wsClient';
import type { Message } from '../types';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import styles from './ChatArea.module.css';

export default function ChatArea() {
  const {
    currentChannelId,
    messages,
    messagesLoading,
    hasMoreMessages,
    setMessages,
    prependMessages,
    setMessagesLoading,
    setHasMoreMessages,
    channels,
    currentView,
    toggleMemberList,
  } = useUiStore();

  const user = useAuthStore((s) => s.user);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const messagesContainerRef = useRef<HTMLDivElement>(null);
  const [input, setInput] = useState('');
  const [typingUsers, setTypingUsers] = useState<string[]>([]);
  const typingTimeoutRef = useRef<ReturnType<typeof setTimeout>>();

  const channel = channels.find((c) => c.id === currentChannelId);

  // Charger les messages quand on change de canal
  useEffect(() => {
    if (!currentChannelId || currentView !== 'chat') return;
    setMessagesLoading(true);
    messagesApi.list(currentChannelId).then((msgs) => {
      setMessages(msgs);
      setHasMoreMessages(msgs.length >= 50);
      setMessagesLoading(false);
      // Scroll en bas
      setTimeout(() => messagesEndRef.current?.scrollIntoView(), 50);
    }).catch(() => setMessagesLoading(false));
  }, [currentChannelId, currentView, setMessages, setHasMoreMessages, setMessagesLoading]);

  // Écouter les indicateurs de frappe
  useEffect(() => {
    const unsub = wsClient.on('TYPING', (data) => {
      const d = data as Record<string, unknown>;
      if (d.channel_id !== currentChannelId) return;
      const username = d.username as string;
      if (username === user?.username) return;
      setTypingUsers((prev) => (prev.includes(username) ? prev : [...prev, username]));
      // Retirer après 3 secondes
      setTimeout(() => {
        setTypingUsers((prev) => prev.filter((u) => u !== username));
      }, 3000);
    });
    return unsub;
  }, [currentChannelId, user?.username]);

  // Charger les messages plus anciens
  const loadOlder = useCallback(() => {
    if (!currentChannelId || messagesLoading || !hasMoreMessages) return;
    const oldest = messages[0];
    if (!oldest) return;
    setMessagesLoading(true);
    messagesApi.list(currentChannelId, oldest.id).then((older) => {
      prependMessages(older);
      setHasMoreMessages(older.length >= 50);
      setMessagesLoading(false);
    }).catch(() => setMessagesLoading(false));
  }, [currentChannelId, messagesLoading, hasMoreMessages, messages, prependMessages, setHasMoreMessages, setMessagesLoading]);

  // Envoyer un message
  const handleSend = () => {
    if (!input.trim() || !currentChannelId) return;
    wsClient.send({
      type: 'MESSAGE_SEND',
      channel_id: currentChannelId,
      content: input.trim(),
    });
    setInput('');
  };

  // Gestion du clavier dans la zone de saisie
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    } else {
      // Envoyer typing
      if (typingTimeoutRef.current) return;
      wsClient.send({
        type: 'TYPING_START',
        channel_id: currentChannelId!,
      });
      typingTimeoutRef.current = setTimeout(() => {
        typingTimeoutRef.current = undefined;
      }, 2000);
    }
  };

  // Auto-resize du textarea
  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    const el = e.target;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 120) + 'px';
  };

  // Ajouter une réaction
  const handleReaction = (messageId: string, emoji: string) => {
    wsClient.send({
      type: 'REACTION_ADD',
      channel_id: currentChannelId!,
      message_id: messageId,
      emoji,
    });
  };

  // Aucun canal sélectionné
  if (!currentChannelId || currentView !== 'chat') {
    return (
      <div className={styles.area}>
        <div className={styles.placeholder}>
          {currentView === 'chat'
            ? 'Sélectionnez un canal pour commencer'
            : null}
        </div>
      </div>
    );
  }

  // Déterminer si un message est une suite (même auteur, < 5min)
  const isCompact = (msg: Message, idx: number) => {
    if (idx === 0) return false;
    const prev = messages[idx - 1];
    if (prev.author_id !== msg.author_id) return false;
    const diff = new Date(msg.created_at).getTime() - new Date(prev.created_at).getTime();
    return diff < 5 * 60 * 1000;
  };

  return (
    <div className={styles.area}>
      {/* En-tête du canal */}
      <div className={styles.header}>
        <span className={styles.headerIcon}>#</span>
        <span className={styles.headerName}>{channel?.name ?? ''}</span>
        {channel?.topic && <span className={styles.headerTopic}>{channel.topic}</span>}
        <div className={styles.headerActions}>
          <button className={styles.headerBtn} onClick={toggleMemberList} title="Membres">
            {'\u{1F465}'}
          </button>
        </div>
      </div>

      {/* Messages */}
      <div className={styles.messages} ref={messagesContainerRef}>
        {hasMoreMessages && messages.length > 0 && (
          <div className={styles.loadMore}>
            <button className={styles.loadMoreBtn} onClick={loadOlder} disabled={messagesLoading}>
              {messagesLoading ? 'Chargement...' : 'Charger les anciens messages'}
            </button>
          </div>
        )}

        {messages.map((msg, idx) => (
          <MessageItem
            key={msg.id}
            message={msg}
            compact={isCompact(msg, idx)}
            isOwn={msg.author_id === user?.id}
            onReaction={(emoji) => handleReaction(msg.id, emoji)}
          />
        ))}

        <div ref={messagesEndRef} />
      </div>

      {/* Indicateur de frappe */}
      <div className={styles.typing}>
        {typingUsers.length > 0 && (
          <>
            <strong>{typingUsers.join(', ')}</strong>
            {typingUsers.length === 1 ? ' est en train d\u2019écrire...' : ' sont en train d\u2019écrire...'}
          </>
        )}
      </div>

      {/* Zone de saisie */}
      <div className={styles.inputArea}>
        <div className={styles.inputWrapper}>
          <button className={styles.attachBtn} title="Joindre un fichier">+</button>
          <textarea
            className={styles.textInput}
            placeholder={`Envoyer un message dans #${channel?.name ?? ''}...`}
            value={input}
            onChange={handleInput}
            onKeyDown={handleKeyDown}
            rows={1}
          />
          <button
            className={styles.sendBtn}
            onClick={handleSend}
            disabled={!input.trim()}
            title="Envoyer"
          >
            \u27A4
          </button>
        </div>
      </div>
    </div>
  );
}

// Composant message individuel
function MessageItem({
  message,
  compact,
  isOwn,
  onReaction,
}: {
  message: Message;
  compact: boolean;
  isOwn: boolean;
  onReaction: (emoji: string) => void;
}) {
  const formatTime = (iso: string) => {
    const d = new Date(iso);
    return d.toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' });
  };

  const formatDate = (iso: string) => {
    const d = new Date(iso);
    const today = new Date();
    if (d.toDateString() === today.toDateString()) {
      return `Aujourd'hui à ${formatTime(iso)}`;
    }
    return d.toLocaleDateString('fr-FR', { day: 'numeric', month: 'short' }) + ` à ${formatTime(iso)}`;
  };

  // Supprimer un message
  const handleDelete = () => {
    wsClient.send({
      type: 'MESSAGE_DELETE',
      channel_id: message.channel_id,
      message_id: message.id,
    });
  };

  // Emojis rapides courants
  const quickEmojis = ['\u{1F44D}', '\u2764\uFE0F', '\u{1F602}', '\u{1F389}'];

  if (compact) {
    return (
      <div className={styles.messageCompact} style={{ position: 'relative' }}>
        <div className={styles.messageBody}>
          <div className={styles.messageContent}>
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{message.content}</ReactMarkdown>
          </div>
          {message.attachment_url && (
            <div className={styles.messageAttachment}>
              {isImageUrl(message.attachment_url) ? (
                <img src={message.attachment_url} alt="pièce jointe" />
              ) : (
                <a href={message.attachment_url} target="_blank" rel="noopener noreferrer">
                  Pièce jointe
                </a>
              )}
            </div>
          )}
          {message.reactions.length > 0 && (
            <div className={styles.reactions}>
              {message.reactions.map((r) => (
                <span key={r.emoji} className={styles.reaction} onClick={() => onReaction(r.emoji)}>
                  {r.emoji} <span className={styles.reactionCount}>{r.count}</span>
                </span>
              ))}
            </div>
          )}
        </div>
        <div className={styles.messageActions}>
          {quickEmojis.map((e) => (
            <button key={e} className={styles.messageActionBtn} onClick={() => onReaction(e)}>{e}</button>
          ))}
          {isOwn && (
            <button className={styles.messageActionBtn} onClick={handleDelete} title="Supprimer">
              {'\u{1F5D1}'}
            </button>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className={styles.message} style={{ position: 'relative' }}>
      <div className={styles.messageAvatar}>
        {message.author_avatar ? (
          <img src={message.author_avatar} alt={message.author_username} />
        ) : (
          message.author_username.charAt(0).toUpperCase()
        )}
      </div>
      <div className={styles.messageBody}>
        <div className={styles.messageHeader}>
          <span className={styles.messageAuthor}>{message.author_username}</span>
          <span className={styles.messageTime}>{formatDate(message.created_at)}</span>
          {message.edited_at && <span className={styles.messageEdited}>(modifié)</span>}
        </div>
        <div className={styles.messageContent}>
          <ReactMarkdown remarkPlugins={[remarkGfm]}>{message.content}</ReactMarkdown>
        </div>
        {message.attachment_url && (
          <div className={styles.messageAttachment}>
            {isImageUrl(message.attachment_url) ? (
              <img src={message.attachment_url} alt="pièce jointe" />
            ) : (
              <a href={message.attachment_url} target="_blank" rel="noopener noreferrer">
                Pièce jointe
              </a>
            )}
          </div>
        )}
        {message.reactions.length > 0 && (
          <div className={styles.reactions}>
            {message.reactions.map((r) => (
              <span key={r.emoji} className={styles.reaction} onClick={() => onReaction(r.emoji)}>
                {r.emoji} <span className={styles.reactionCount}>{r.count}</span>
              </span>
            ))}
          </div>
        )}
      </div>
      <div className={styles.messageActions}>
        {quickEmojis.map((e) => (
          <button key={e} className={styles.messageActionBtn} onClick={() => onReaction(e)}>{e}</button>
        ))}
        {isOwn && (
          <button className={styles.messageActionBtn} onClick={handleDelete} title="Supprimer">
            {'\u{1F5D1}'}
          </button>
        )}
      </div>
    </div>
  );
}

// Vérifier si une URL est une image
function isImageUrl(url: string): boolean {
  return /\.(png|jpe?g|gif|webp|svg)(\?.*)?$/i.test(url);
}
