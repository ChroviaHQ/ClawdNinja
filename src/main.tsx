import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { api } from './api';
import { App } from './App';
import { useStore } from './store';
import './styles.css';

void api.getPort().then(port => {
  useStore.getState().setPort(String(port));
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
});
