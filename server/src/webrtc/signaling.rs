// Relai de signaling WebRTC — les données SDP/ICE sont relayées telles quelles via WebSocket
// Ce module est volontairement minimal car toute la logique de relai
// est gérée dans ws/handlers.rs (événement WEBRTC_SIGNAL)
//
// Le flux WebRTC fonctionne ainsi :
// 1. Client A envoie VOICE_JOIN pour un salon vocal
// 2. Le serveur notifie les autres membres (VOICE_USER_JOINED)
// 3. Client A crée un RTCPeerConnection et envoie une offre SDP via WEBRTC_SIGNAL
// 4. Le serveur relaie le signal au client cible
// 5. Le client cible répond avec une réponse SDP
// 6. Les candidats ICE sont échangés de la même manière
// 7. La connexion P2P est établie directement entre les clients
