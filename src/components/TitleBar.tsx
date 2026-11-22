import { getCurrentWindow } from '@tauri-apps/api/window';
import { ArrowUpCircle, Loader2, Minus, X } from 'lucide-react';
import { useEffect, useState } from 'react';

import { version } from '../../package.json';
import { api, type TitleBarMetrics } from '../api';
import { interpolate, t } from '../i18n';
import { isMac } from '../platform';
import { useStore } from '../store';
import { Logo } from './Logo';
import { IconButton } from './ui';

function WindowControls() {
  const appWindow = getCurrentWindow();
  return (
    <span className='-mr-2.5 flex gap-0.5'>
      <IconButton title={t.minimize} onClick={() => appWindow.minimize()}>
        <Minus className='size-3.5' />
      </IconButton>
      <IconButton
        title={t.close}
        onClick={() => appWindow.close()}
        className='hover:bg-[#c42b1c] hover:text-white'
      >
        <X className='size-3.5' />
      </IconButton>
    </span>
  );
}

function UpdateButton({version}: {version: string;}) {
  const installUpdate = useStore(s => s.installUpdate);
  const [updating, setUpdating] = useState(false);
  const install = async () => {
    setUpdating(true);
    await installUpdate();
    setUpdating(false);
  };
  return (
    <button
      type='button'
      title={t.updateHint}
      disabled={updating}
      onClick={install}
      className='flex h-6 items-center gap-1 rounded-md border border-line px-2 text-[11.5px] font-medium text-ok hover:enabled:bg-surface-2 disabled:opacity-60'
    >
      {updating
        ? <Loader2 className='size-3.5 animate-spin' />
        : <ArrowUpCircle className='size-3.5' />}
      {updating ? t.updating : interpolate(t.updateTo, {version})}
    </button>
  );
}

function useTitleBarMetrics(enabled: boolean): TitleBarMetrics | null {
  const [metrics, setMetrics] = useState<TitleBarMetrics | null>(null);
  useEffect(() => {
    if (!enabled) {
      return;
    }
    void api.getTitleBarMetrics().then(value => value && setMetrics(value), () => {});
    const unlisten = api.onTitleBarMetrics(setMetrics);
    return () => void unlisten.then(dispose => dispose());
  }, [enabled]);
  return enabled ? metrics : null;
}

export function TitleBar() {
  const updateVersion = useStore(s => s.updateVersion);
  const popover = useStore(s => s.popover);
  const metrics = useTitleBarMetrics(isMac && !popover);
  const drag = popover ? {} : {'data-tauri-drag-region': true};
  return (
    <header
      {...drag}
      className='flex h-10 shrink-0 items-center gap-3 px-4.5'
      style={metrics ? {height: metrics.height} : undefined}
    >
      {isMac && !popover && (
        <span
          data-tauri-drag-region
          className='-ml-4.5 w-19.5 shrink-0 self-stretch'
          style={metrics ? {width: metrics.controlsWidth} : undefined}
        />
      )}
      <span {...drag} className='pointer-events-none flex items-center gap-1.5'>
        <Logo />
        <h1 className='m-0 text-[13px] font-[650] tracking-tight'>ClawdNinja</h1>
      </span>
      <span {...drag} className='ml-auto flex items-center gap-1.5'>
        {updateVersion
          ? <UpdateButton version={updateVersion} />
          : <span {...drag} className='text-[11.5px] text-muted tabular-nums'>v{version}</span>}
        {!isMac && !popover && <WindowControls />}
      </span>
    </header>
  );
}
