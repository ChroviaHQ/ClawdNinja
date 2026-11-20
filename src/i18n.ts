import type { CommandError } from './api';
import en from './locales/en.json';
import zh from './locales/zh.json';
import { isMac } from './platform';

type Messages = typeof en;

const isZh = navigator.language.toLowerCase().startsWith('zh');
export const t: Messages = isZh ? zh : en;
document.documentElement.lang = isZh ? 'zh-CN' : 'en';

export function interpolate(template: string, values: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (match, key: string) => String(values[key] ?? match));
}

export function describeError(
  error: CommandError,
  context: {name?: string; port?: number;} = {},
): string {
  const errors: Record<string, string> = t.errors;
  return interpolate(errors[error.code] ?? errors.unknown, {
    name: context.name ?? '',
    port: context.port ?? '',
    detail: error.detail,
    quitHint: isMac ? t.quitHintMac : t.quitHintWindows,
  }).trim();
}
