// Module WebRTC — le signaling passe par WebSocket
// Le serveur ne fait que relayer les offres SDP et candidats ICE entre pairs
// Voir ws/handlers.rs pour l'implémentation du relai WEBRTC_SIGNAL
pub mod signaling;
