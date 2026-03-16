// Client WebSocket singleton — gère la connexion, reconnexion et dispatch des événements
import type { WsMessage, ServerEventType } from '../types';
import { API_BASE_URL } from '../config';

type EventHandler = (data: WsMessage) => void;

class WsClient {
  private ws: WebSocket | null = null;
  private token: string | null = null;
  private listeners = new Map<string, Set<EventHandler>>();
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectDelay = 1000;
  private maxReconnectDelay = 30000;
  private intentionalClose = false;

  // Connexion au serveur WebSocket
  connect(token: string): void {
    this.token = token;
    this.intentionalClose = false;

    let url: string;
    if (API_BASE_URL) {
      // Mode Tauri ou URL explicite — construire le WS à partir de l'URL configurée
      const base = API_BASE_URL.replace(/^http/, 'ws');
      url = `${base}/ws`;
    } else {
      // Mode dev Vite — utiliser window.location (le proxy redirige /ws)
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      url = `${protocol}//${window.location.host}/ws`;
    }

    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      this.reconnectDelay = 1000;
      // Envoyer l'authentification dès la connexion
      this.send({ type: 'AUTH', token: this.token! });
    };

    this.ws.onmessage = (event) => {
      try {
        const data: WsMessage = JSON.parse(event.data);
        this.dispatch(data.type as string, data);
      } catch {
        console.error('[WS] Message invalide:', event.data);
      }
    };

    this.ws.onclose = () => {
      this.ws = null;
      if (!this.intentionalClose) {
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = () => {
      // onclose sera appelé juste après
    };
  }

  // Déconnexion volontaire
  disconnect(): void {
    this.intentionalClose = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  // Envoyer un message au serveur
  send(message: WsMessage): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  // S'abonner à un type d'événement
  on(eventType: ServerEventType | string, handler: EventHandler): () => void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }
    this.listeners.get(eventType)!.add(handler);

    // Retourne une fonction de désabonnement
    return () => {
      this.listeners.get(eventType)?.delete(handler);
    };
  }

  // Retirer un handler
  off(eventType: string, handler: EventHandler): void {
    this.listeners.get(eventType)?.delete(handler);
  }

  // Dispatch interne
  private dispatch(eventType: string, data: WsMessage): void {
    const handlers = this.listeners.get(eventType);
    if (handlers) {
      handlers.forEach((handler) => handler(data));
    }
    // Aussi notifier les listeners '*' (wildcard)
    const wildcardHandlers = this.listeners.get('*');
    if (wildcardHandlers) {
      wildcardHandlers.forEach((handler) => handler(data));
    }
  }

  // Reconnexion avec backoff exponentiel
  private scheduleReconnect(): void {
    if (this.reconnectTimer) return;

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      if (this.token) {
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
        this.connect(this.token);
      }
    }, this.reconnectDelay);
  }

  get connected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}

// Singleton exporté
export const wsClient = new WsClient();
