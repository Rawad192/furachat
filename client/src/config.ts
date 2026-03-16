// Configuration client — URL du serveur backend
// En dev (Vite), le proxy s'occupe de rediriger /api et /ws vers le backend.
// En desktop (Tauri), VITE_API_URL doit pointer vers le serveur réel (ex: http://localhost:8080).
export const API_BASE_URL: string = import.meta.env.VITE_API_URL || '';
