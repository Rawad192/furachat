# Tutoriel — Lancer FuraChat

Guide détaillé pour installer, configurer et exécuter FuraChat (serveur + client).

---

## 1. Prérequis

Avant de commencer, assurez-vous d'avoir installé les outils suivants sur votre machine.

### Rust (pour le serveur backend)

```bash
# Installer Rust via rustup (Linux/macOS)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Suivre les instructions à l'écran, puis recharger votre shell
source ~/.cargo/env

# Vérifier l'installation
rustc --version    # doit afficher >= 1.75
cargo --version
```

Sur **Windows**, téléchargez l'installeur depuis https://rustup.rs.

### Node.js (pour le client frontend)

```bash
# Option 1 : via nvm (recommandé)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
source ~/.bashrc
nvm install 18
nvm use 18

# Option 2 : via votre gestionnaire de paquets
# Ubuntu/Debian :
sudo apt install nodejs npm

# macOS :
brew install node

# Vérifier l'installation
node --version    # doit afficher >= 18
npm --version
```

### Tauri CLI (optionnel — uniquement pour la version desktop)

```bash
cargo install tauri-cli
```

---

## 2. Cloner le projet

```bash
git clone https://github.com/Rawad192/furachat.git
cd furachat
```

---

## 3. Lancer le serveur backend

### 3.1 Se placer dans le répertoire serveur

```bash
cd server
```

### 3.2 Compiler le serveur

La première compilation télécharge les dépendances et compile SQLite (bundled).
Cela peut prendre 2 à 5 minutes la première fois.

```bash
cargo build --release
```

Si vous préférez compiler en mode debug (plus rapide à compiler, plus lent à exécuter) :

```bash
cargo build
```

### 3.3 Configurer les variables d'environnement (optionnel)

Le serveur fonctionne avec des valeurs par défaut. Vous pouvez les personnaliser :

```bash
# Port du serveur (défaut : 8080)
export FURACHAT_PORT=8080

# Adresse d'écoute (défaut : 0.0.0.0 = toutes les interfaces)
export FURACHAT_HOST=0.0.0.0

# Secret JWT — IMPORTANT : changez ceci en production !
export FURACHAT_JWT_SECRET="mon-secret-tres-long-et-securise"

# Répertoire de données (défaut : data/)
export FURACHAT_DATA_DIR=data

# Durée de validité des tokens JWT en secondes (défaut : 604800 = 7 jours)
export FURACHAT_JWT_EXPIRATION=604800

# Seuil d'espace disque pour l'archivage automatique en octets (défaut : 500 Mo)
export FURACHAT_ARCHIVE_THRESHOLD=524288000

# Taille maximale d'upload en octets (défaut : 50 Mo)
export FURACHAT_MAX_UPLOAD=52428800
```

Vous pouvez aussi créer un fichier `.env` ou un script shell pour charger ces variables.

### 3.4 Lancer le serveur

```bash
# Mode release (recommandé pour une utilisation normale)
cargo run --release

# OU en mode debug (pour le développement)
cargo run
```

Vous devriez voir dans le terminal :

```
[INFO] Configuration chargée: port=8080, host=0.0.0.0
[INFO] Base de données initialisée: data/furachat.db
[INFO] Migrations appliquées
[INFO] Serveur FuraChat démarré sur 0.0.0.0:8080
```

### 3.5 Vérifier que le serveur fonctionne

Dans un autre terminal :

```bash
curl http://localhost:8080/api/health
```

Réponse attendue :

```json
{"status":"ok","uptime_seconds":5}
```

### 3.6 Structure des données créées automatiquement

Au premier lancement, le serveur crée :

```
server/data/
├── furachat.db       ← Base de données SQLite
├── avatars/          ← Avatars des utilisateurs
├── banners/          ← Bannières (utilisateurs et serveurs)
├── files/            ← Fichiers uploadés
├── stickers/         ← Stickers des serveurs
├── badges/           ← Icônes de badges
└── archives/         ← Archives de messages (archivage automatique)
```

---

## 4. Lancer le client frontend

Ouvrez un **nouveau terminal** (gardez le serveur en cours d'exécution).

### 4.1 Se placer dans le répertoire client

```bash
cd furachat/client
```

### 4.2 Installer les dépendances

```bash
npm install
```

### 4.3 Lancer le serveur de développement

```bash
npm run dev
```

Vous verrez :

```
  VITE v5.x.x  ready in XXX ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: http://192.168.x.x:5173/
```

### 4.4 Ouvrir l'application

Ouvrez votre navigateur et accédez à :

```
http://localhost:5173
```

Le proxy Vite redirige automatiquement :
- Les requêtes `/api/*` vers le serveur Rust (`localhost:8080`)
- Les connexions `/ws` vers le WebSocket du serveur

### 4.5 Première utilisation

1. **Créer un compte** : cliquez sur « Créer un compte », remplissez le formulaire (nom d'utilisateur, e-mail, mot de passe)
2. **Créer un serveur** : cliquez sur le bouton « + » dans la barre de serveurs en haut
3. **Envoyer un message** : sélectionnez le canal #général, tapez votre message et appuyez sur Entrée
4. **Inviter des amis** : allez dans les paramètres du serveur (icône ⚙) → onglet « Invitations » → « Générer une invitation » → partagez le code

---

## 5. Builder pour la production

### 5.1 Builder le frontend

```bash
cd furachat/client
npm run build
```

Les fichiers statiques sont générés dans `client/dist/`.

### 5.2 Servir le frontend depuis le backend

En production, vous pouvez servir les fichiers statiques directement depuis le serveur Rust.
Copiez le dossier `client/dist/` dans `server/data/static/` et configurez un reverse proxy
(Nginx, Caddy) ou servez-les via Tower (déjà disponible dans le serveur).

### 5.3 Exemple avec Nginx

```nginx
server {
    listen 80;
    server_name furachat.example.com;

    # Frontend statique
    location / {
        root /chemin/vers/furachat/client/dist;
        try_files $uri $uri/ /index.html;
    }

    # API backend
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # WebSocket
    location /ws {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # Fichiers uploadés
    location /api/files/ {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

---

## 6. Lancer la version desktop (Tauri)

### 6.1 Prérequis supplémentaires

Sur **Linux**, installez les dépendances système Tauri :

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

Sur **macOS** : Xcode Command Line Tools suffit.

Sur **Windows** : installez Microsoft Visual Studio Build Tools + WebView2.

### 6.2 Lancer en mode développement

```bash
cd furachat/client

# Le serveur backend doit être en cours d'exécution séparément
npx tauri dev
```

Cela ouvre une fenêtre desktop native avec l'application FuraChat.

### 6.3 Compiler l'exécutable

```bash
npx tauri build
```

L'exécutable est généré dans `client/src-tauri/target/release/`.

---

## 7. Résolution de problèmes

### Le serveur ne démarre pas

```
Error: Address already in use
```
→ Un autre processus utilise le port 8080. Changez le port :
```bash
export FURACHAT_PORT=9090
cargo run --release
```

### Le client n'arrive pas à se connecter au serveur

1. Vérifiez que le serveur backend est bien lancé (`curl http://localhost:8080/api/health`)
2. Vérifiez que le port correspond dans `vite.config.ts` (proxy vers `localhost:8080`)
3. Si le serveur est sur un autre port, modifiez le proxy dans `client/vite.config.ts`

### Erreur de compilation Rust

```bash
# Mettre à jour Rust
rustup update

# Nettoyer le cache de compilation
cargo clean
cargo build --release
```

### Erreur npm install

```bash
# Supprimer node_modules et réinstaller
rm -rf node_modules package-lock.json
npm install
```

### La base de données est corrompue

```bash
# Supprimer la base et relancer (perte de toutes les données)
rm server/data/furachat.db
cargo run --release
```

---

## 8. Résumé des commandes

| Action | Commande |
|--------|----------|
| Compiler le serveur | `cd server && cargo build --release` |
| Lancer le serveur | `cd server && cargo run --release` |
| Installer les dépendances client | `cd client && npm install` |
| Lancer le client (dev) | `cd client && npm run dev` |
| Builder le client | `cd client && npm run build` |
| Lancer en desktop | `cd client && npx tauri dev` |
| Builder le desktop | `cd client && npx tauri build` |
| Vérifier le serveur | `curl http://localhost:8080/api/health` |

---

## 9. Ports utilisés

| Service | Port | URL |
|---------|------|-----|
| Backend Rust | 8080 | `http://localhost:8080` |
| Frontend Vite (dev) | 5173 | `http://localhost:5173` |
| WebSocket | 8080 | `ws://localhost:8080/ws` |

Le client de développement (port 5173) redirige automatiquement les appels API et WebSocket
vers le serveur backend (port 8080) grâce au proxy configuré dans Vite.

En production, un reverse proxy (Nginx/Caddy) unifie tout sur un seul port (80/443).
