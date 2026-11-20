import {
  Download,
  Eraser,
  ExternalLink,
  FileCog,
  FlaskConical,
  Monitor,
  Play,
  Settings2,
  Terminal,
} from 'lucide-react';

import { api, type DesktopApp } from './api';
import { OpenAI, VisualStudioCode } from './components/BrandIcons';
import type { IconComponent } from './components/ui';
import { interpolate, t } from './i18n';

export type ItemId = DesktopApp | 'cli' | 'settings';

export type Action = {
  label: string;
  icon: IconComponent;
  needsPort: boolean;
  primary?: boolean;
  run: (port: number) => Promise<string>;
};

export type Item = {
  id: ItemId;
  name: string;
  icon: IconComponent;
  probe: () => Promise<boolean>;
  on: string;
  off: string;
  whenOn: Action;
  whenOff: Action;
};

function desktopApp(id: DesktopApp, name: string, icon: IconComponent): Item {
  return {
    id,
    name,
    icon,
    probe: () => api.isAppInstalled(id),
    on: t.installed,
    off: t.notInstalled,
    whenOn: {
      label: t.launch,
      icon: Play,
      needsPort: true,
      primary: true,
      run: async port => {
        await api.launchApp(id, port);
        return interpolate(t.results.launched, {name});
      },
    },
    whenOff: {
      label: t.download,
      icon: ExternalLink,
      needsPort: false,
      run: async () =>
        interpolate(t.results.downloadOpened, {name, url: await api.openAppDownload(id)}),
    },
  };
}

export const ITEMS: Item[] = [
  desktopApp('claude', 'Claude Desktop', Monitor),
  desktopApp('science', 'Claude Science', FlaskConical),
  desktopApp('chatgpt', 'ChatGPT', OpenAI),
  desktopApp('vscode', 'Visual Studio Code', VisualStudioCode),
  {
    id: 'cli',
    name: 'Claude Code CLI',
    icon: Terminal,
    probe: api.isCliInstalled,
    on: t.installed,
    off: t.notInstalled,
    whenOn: {
      label: t.launch,
      icon: Play,
      needsPort: true,
      primary: true,
      run: async port => {
        await api.launchCli(port);
        return t.results.terminalOpened;
      },
    },
    whenOff: {
      label: t.install,
      icon: Download,
      needsPort: true,
      run: async port => {
        await api.installCli(port);
        return t.results.cliInstalled;
      },
    },
  },
  {
    id: 'settings',
    name: t.claudeCodeSettings,
    icon: FileCog,
    probe: api.isProxyApplied,
    on: t.egressApplied,
    off: t.egressNotApplied,
    whenOn: {
      label: t.remove,
      icon: Eraser,
      needsPort: false,
      run: async () => {
        await api.removeProxy();
        return t.results.egressRemoved;
      },
    },
    whenOff: {
      label: t.apply,
      icon: Settings2,
      needsPort: true,
      run: async port => interpolate(t.results.egressApplied, {path: await api.applyProxy(port)}),
    },
  },
];
