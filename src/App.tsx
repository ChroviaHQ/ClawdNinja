import { useEffect } from 'react';

import { EgressCard } from './components/EgressCard';
import { NoteToast } from './components/NoteToast';
import { ProductCard } from './components/ProductCard';
import { TitleBar } from './components/TitleBar';
import { useStore } from './store';

export function App() {
  const init = useStore(s => s.init);

  useEffect(() => {
    const cleanup = init();
    return () => void cleanup.then(dispose => dispose());
  }, [init]);

  return (
    <div className='flex h-full flex-col'>
      <TitleBar />
      <main className='flex flex-1 flex-col gap-3 overflow-y-auto px-4.5 pb-4.5'>
        <EgressCard />
        <ProductCard />
      </main>
      <NoteToast />
    </div>
  );
}
