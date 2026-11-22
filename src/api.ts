import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type DesktopApp = 'claude' | 'science' | 'chatgpt' | 'vscode' | 'antigravity';

export type InstallStage = 'script' | 'download' | 'setup';

export type TitleBarMetrics = {height: number; controlsWidth: number;};

export type CommandError = {code: string; detail: string;};

export function toCommandError(error: unknown): CommandError {
  if (error && typeof error === 'object' && 'code' in error) {
    return error as CommandError;
  }
  return {code: 'unknown', detail: String(error)};
}

function listenPayload<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, ({payload}) => handler(payload));
}

export const api = {
  getPort: () => invoke<number>('get_port'),
  isAppInstalled: (app: DesktopApp) => invoke<boolean>('is_app_installed', {app}),
  launchApp: (app: DesktopApp, port: number) => invoke<void>('launch_app', {app, port}),
  openAppDownload: (app: DesktopApp) => invoke<string>('open_app_download', {app}),
  isCliInstalled: () => invoke<boolean>('is_cli_installed'),
  launchCli: (port: number) => invoke<void>('launch_cli', {port}),
  installCli: (port: number) => invoke<void>('install_cli', {port}),
  isProxyApplied: () => invoke<boolean>('is_proxy_applied'),
  applyProxy: (port: number) => invoke<string>('apply_proxy', {port}),
  removeProxy: () => invoke<void>('remove_proxy'),
  getTitleBarMetrics: () => invoke<TitleBarMetrics | null>('get_title_bar_metrics'),
  isPopover: () => invoke<boolean>('is_popover'),
  getPendingUpdate: () => invoke<string | null>('get_pending_update'),
  installUpdate: () => invoke<void>('install_update'),
  onInstallStage: (handler: (stage: InstallStage) => void) =>
    listenPayload('install://stage', handler),
  onUpdate: (handler: (version: string) => void) => listenPayload('app://update', handler),
  onPopover: (handler: (popover: boolean) => void) => listenPayload('window://popover', handler),
  onTitleBarMetrics: (handler: (metrics: TitleBarMetrics) => void) =>
    listenPayload('window://title-bar-metrics', handler),
};
