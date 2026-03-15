// Vue Forum — liste de posts, création et réponses
import { useEffect, useState } from 'react';
import { useUiStore } from '../stores/uiStore';
import { forum as forumApi } from '../services/api';
import type { ForumPost, ForumReply } from '../types';
import styles from './ForumView.module.css';

export default function ForumView() {
  const { currentChannelId, forumPosts, setForumPosts, channels } = useUiStore();
  const [selectedPost, setSelectedPost] = useState<ForumPost | null>(null);
  const [showCreate, setShowCreate] = useState(false);

  const channel = channels.find((c) => c.id === currentChannelId);

  // Charger les posts au montage
  useEffect(() => {
    if (!currentChannelId) return;
    forumApi.listPosts(currentChannelId).then(setForumPosts).catch(console.error);
  }, [currentChannelId, setForumPosts]);

  if (!currentChannelId) {
    return (
      <div className={styles.container}>
        <div className={styles.empty}>Sélectionnez un canal forum</div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <span className={styles.headerIcon}>{'\u{1F4CB}'}</span>
        <span className={styles.headerName}>{channel?.name ?? 'Forum'}</span>
      </div>

      <div className={styles.content}>
        {selectedPost ? (
          <PostDetail
            post={selectedPost}
            channelId={currentChannelId}
            onBack={() => setSelectedPost(null)}
          />
        ) : (
          <>
            {showCreate ? (
              <CreatePostForm
                channelId={currentChannelId}
                onCreated={(post) => {
                  setForumPosts([post, ...forumPosts]);
                  setShowCreate(false);
                }}
                onCancel={() => setShowCreate(false)}
              />
            ) : (
              <button className={styles.newPostBtn} onClick={() => setShowCreate(true)}>
                Nouveau post
              </button>
            )}

            {forumPosts.length === 0 && !showCreate && (
              <div className={styles.empty}>Aucun post dans ce forum</div>
            )}

            <div className={styles.postList}>
              {forumPosts.map((post) => (
                <div key={post.id} className={styles.postCard} onClick={() => setSelectedPost(post)}>
                  <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                    {post.pinned && <span className={styles.postPinned}>{'\u{1F4CC}'}</span>}
                    <div className={styles.postTitle}>{post.title}</div>
                  </div>
                  <div className={styles.postMeta}>
                    <span>{post.author_username}</span>
                    <span>{post.reply_count} réponse{post.reply_count !== 1 ? 's' : ''}</span>
                    <span>{new Date(post.created_at).toLocaleDateString('fr-FR')}</span>
                  </div>
                </div>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

// Formulaire de création de post
function CreatePostForm({
  channelId,
  onCreated,
  onCancel,
}: {
  channelId: string;
  onCreated: (post: ForumPost) => void;
  onCancel: () => void;
}) {
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async () => {
    if (!title.trim() || !content.trim()) return;
    setSubmitting(true);
    try {
      const post = await forumApi.createPost(channelId, { title: title.trim(), content: content.trim() });
      onCreated(post);
    } catch (e) {
      console.error('Erreur création post:', e);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className={styles.form}>
      <input
        className={styles.formInput}
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        placeholder="Titre du post"
        maxLength={200}
      />
      <textarea
        className={styles.formTextarea}
        value={content}
        onChange={(e) => setContent(e.target.value)}
        placeholder="Contenu..."
      />
      <div style={{ display: 'flex', gap: 12 }}>
        <button className={styles.formBtn} onClick={handleSubmit} disabled={submitting}>
          {submitting ? 'Publication...' : 'Publier'}
        </button>
        <button onClick={onCancel} style={{ padding: '10px 20px', color: 'var(--text-muted)' }}>
          Annuler
        </button>
      </div>
    </div>
  );
}

// Vue détaillée d'un post + réponses
function PostDetail({
  post,
  channelId,
  onBack,
}: {
  post: ForumPost;
  channelId: string;
  onBack: () => void;
}) {
  const [replies, setReplies] = useState<ForumReply[]>([]);
  const [replyText, setReplyText] = useState('');
  const [sending, setSending] = useState(false);

  useEffect(() => {
    forumApi.getReplies(channelId, post.id).then(setReplies).catch(console.error);
  }, [channelId, post.id]);

  const handleReply = async () => {
    if (!replyText.trim()) return;
    setSending(true);
    try {
      const reply = await forumApi.createReply(channelId, post.id, { content: replyText.trim() });
      setReplies([...replies, reply]);
      setReplyText('');
    } catch (e) {
      console.error('Erreur réponse:', e);
    } finally {
      setSending(false);
    }
  };

  return (
    <div className={styles.postDetail}>
      <span className={styles.backLink} onClick={onBack}>\u2190 Retour aux posts</span>

      <div className={styles.postDetailTitle}>{post.title}</div>
      <div className={styles.postDetailMeta}>
        Par <strong>{post.author_username}</strong> le {new Date(post.created_at).toLocaleDateString('fr-FR')}
        {post.locked && ' \u{1F512} Verrouill\u00E9'}
      </div>
      <div className={styles.postDetailContent}>{post.content}</div>

      <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>
        Réponses ({replies.length})
      </h3>

      <div className={styles.replies}>
        {replies.map((reply) => (
          <div key={reply.id} className={styles.reply}>
            <div className={styles.replyAvatar}>
              {reply.author_avatar ? (
                <img src={reply.author_avatar} alt={reply.author_username} />
              ) : (
                reply.author_username.charAt(0).toUpperCase()
              )}
            </div>
            <div className={styles.replyBody}>
              <div>
                <span className={styles.replyAuthor}>{reply.author_username}</span>
                <span className={styles.replyTime}>
                  {new Date(reply.created_at).toLocaleString('fr-FR')}
                </span>
              </div>
              <div className={styles.replyContent}>{reply.content}</div>
            </div>
          </div>
        ))}
      </div>

      {!post.locked && (
        <div className={styles.replyInputArea}>
          <input
            className={styles.replyInput}
            value={replyText}
            onChange={(e) => setReplyText(e.target.value)}
            placeholder="Écrire une réponse..."
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                handleReply();
              }
            }}
          />
          <button className={styles.replySendBtn} onClick={handleReply} disabled={sending}>
            {sending ? '...' : 'Répondre'}
          </button>
        </div>
      )}
    </div>
  );
}
