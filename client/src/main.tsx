// Point d'entrée React pour FuraChat
// Sera complété en Phase 7
import React from 'react';
import ReactDOM from 'react-dom/client';

const root = document.getElementById('root');
if (root) {
  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      <div>FuraChat</div>
    </React.StrictMode>
  );
}
