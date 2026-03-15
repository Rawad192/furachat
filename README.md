# FuraChat v1.0

Messagerie auto-hébergée inspirée de Discord — full-stack Rust + React + Tauri.

## Architecture

```
server/          → Backend Rust (Axum 0.7 + SQLite + WebSocket)
client/          → Frontend React 18 + TypeScript + Vite
client/src-tauri → Wrapper desktop Tauri 2
```

## Fonctionnalités

- **Auth** : inscription/connexion, JWT + Argon2
- **Serveurs** : créer, rejoindre (invitations), icône/bannière, catégories, canaux (texte, vocal, forum, annonce)
- **Messages** : envoi temps réel (WebSocket), Markdown, pièces jointes, réactions, édition/suppression
- **Messages directs** : entre utilisateurs
- **Forum** : posts + réponses dans les canaux forum
- **Vocal** : rejoindre/quitter les salons, signaling WebRTC, stub RNNoise
- **Rôles & Permissions** : système complet avec résolution @everyone → rôles → overrides par canal
- **Modération** : kick, ban/unban, mute/unmute, journal d'audit
- **Profil** : avatar, bannière, bio, statut personnalisé, badges
- **Stickers** : upload et utilisation par serveur
- **Archivage automatique** : export JSON + nettoyage des anciens messages quand le disque est plein
- **Amis** : liste d'amis en barre de contacts

## Prérequis

- **Rust** >= 1.75 (pour le backend)
- **Node.js** >= 18 (pour le frontend)
- **Tauri CLI** (optionnel, pour le desktop)

## Lancement rapide

### Backend

```bash
cd server
# Configurer les variables d'environnement (optionnel — valeurs par défaut incluses)
export JWT_SECRET="votre_secret_jwt"
cargo run
```

Le serveur démarre sur `http://localhost:8080`.

### Frontend (dev)

```bash
cd client
npm install
npm run dev
```

Le client dev tourne sur `http://localhost:5173` avec proxy vers le backend.

### Desktop (Tauri)

```bash
cd client
npm install
npx tauri dev
```

## Variables d'environnement (backend)

| Variable | Défaut | Description |
|----------|--------|-------------|
| `PORT` | `8080` | Port du serveur |
| `HOST` | `0.0.0.0` | Adresse d'écoute |
| `DATABASE_URL` | `data/furachat.db` | Chemin de la base SQLite |
| `JWT_SECRET` | `furachat-dev-secret-change-me` | Secret JWT |
| `DATA_DIR` | `data` | Répertoire de données |
| `ARCHIVE_THRESHOLD_PERCENT` | `90` | Seuil d'archivage (% disque utilisé) |
| `MAX_UPLOAD_SIZE` | `10485760` | Taille max upload (octets) |

## Stack technique

| Composant | Technologie |
|-----------|-------------|
| Backend | Rust, Axum 0.7, Tower, SQLite (rusqlite + r2d2) |
| Auth | JWT (jsonwebtoken), Argon2 |
| Temps réel | WebSocket natif Axum |
| Vocal | WebRTC signaling via WS |
| Frontend | React 18, TypeScript, Vite, Zustand |
| Desktop | Tauri 2 |
| Rendu Markdown | react-markdown + remark-gfm |

## Structure du projet

```
server/src/
├── api/            # Routes REST (~14 modules)
├── auth/           # JWT, Argon2, middleware AuthUser
├── config.rs       # Configuration depuis env
├── db/
│   ├── migrations.rs  # Schéma SQLite (20+ tables)
│   ├── pool.rs        # Pool r2d2
│   └── queries/       # Requêtes par domaine
├── error.rs        # Gestion d'erreurs unifiée
├── models/         # Structs sérialisables
├── storage/        # Archivage automatique
├── webrtc/         # Documentation signaling
├── ws/             # Hub, events, handlers, connexion
└── main.rs         # Point d'entrée

client/src/
├── components/     # Composants React (12 modules)
├── hooks/          # useAuth
├── services/       # API, WebSocket, RNNoise stub
├── stores/         # Zustand (auth, ui)
├── styles/         # CSS global + modules
└── types/          # Types TypeScript
```

## Licence

Projet privé.
