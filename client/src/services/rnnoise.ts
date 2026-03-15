// Stub RNNoise — service complet sans binaire WASM
// Fournit l'interface de suppression de bruit audio

export interface RNNoiseState {
  enabled: boolean;
}

const state: RNNoiseState = { enabled: false };

// Initialisation (stub — pas de WASM chargé)
export async function initRNNoise(): Promise<void> {
  console.info('[RNNoise] Stub initialisé — suppression de bruit simulée');
}

// Activer/désactiver la suppression de bruit
export function setRNNoiseEnabled(enabled: boolean): void {
  state.enabled = enabled;
  console.info(`[RNNoise] ${enabled ? 'Activé' : 'Désactivé'} (stub)`);
}

export function isRNNoiseEnabled(): boolean {
  return state.enabled;
}

// Créer un noeud de traitement audio (passthrough)
export function createRNNoiseProcessor(
  audioContext: AudioContext,
  source: MediaStreamAudioSourceNode
): AudioNode {
  // En mode stub, on retourne directement la source — pas de traitement
  // Quand le vrai WASM sera intégré, on remplacera par un AudioWorkletNode
  const passthrough = audioContext.createGain();
  passthrough.gain.value = 1.0;
  source.connect(passthrough);
  return passthrough;
}

// Nettoyer les ressources
export function destroyRNNoiseProcessor(): void {
  console.info('[RNNoise] Processeur détruit (stub)');
}
