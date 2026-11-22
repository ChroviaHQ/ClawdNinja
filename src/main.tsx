import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { api } from './api';
import { App } from './App';
import { useStore } from './store';
import './styles.css';

void Promise.all([api.getPort(), api.isPopover()]).then(([port, popover]) => {
  useStore.setState({port: String(port), popover});
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
});
