// Composant racine — routage entre login et application principale
import { useAuth } from './hooks/useAuth';
import LoginScreen from './components/LoginScreen';
import MainLayout from './components/MainLayout';

export default function App() {
  const { user, loading } = useAuth();

  // Écran de chargement pendant la restauration de session
  if (loading && !user) {
    return (
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100%',
        color: 'var(--text-muted)',
        fontSize: '16px',
      }}>
        Chargement...
      </div>
    );
  }

  // Non connecté → écran de login
  if (!user) {
    return <LoginScreen />;
  }

  // Connecté → layout principal
  return <MainLayout />;
}
