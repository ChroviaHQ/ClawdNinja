import { create } from 'zustand';

import { api, type CommandError, type InstallStage, toCommandError } from './api';
import { describeError } from './i18n';
import { type Action, type Item, type ItemId, ITEMS } from './items';

const MAX_PORT = 65535;
const SUCCESS_NOTE_MS = 4000;

export type Probe = {status: 'checking';} | {status: 'done'; present: boolean;} | {
  status: 'failed';
  error: CommandError;
};

type Busy = {id: ItemId; startedAt: number; stage?: InstallStage;};

export type Note = {text: string; error: boolean;};

export function parsePort(value: string): number | null {
  const port = Number(value);
  return /^\d{1,5}$/.test(value) && port > 0 && port <= MAX_PORT ? port : null;
}

type Store = {
  port: string;
  probes: Record<ItemId, Probe>;
  busy: Busy | null;
  note: Note | null;
  updateVersion: string | null;
  popover: boolean;
  init: () => Promise<() => void>;
  refresh: () => Promise<void>;
  setPort: (port: string) => void;
  run: (item: Item, action: Action) => Promise<void>;
  installUpdate: () => Promise<void>;
  dismissNote: () => void;
};

const checking = Object.fromEntries(ITEMS.map(item => [item.id, {status: 'checking'}])) as Record<
  ItemId,
  Probe
>;

export const useStore = create<Store>((set, get) => {
  const notify = (note: Note) => {
    set({note});
    if (!note.error) {
      setTimeout(() => {
        if (get().note === note) {
          set({note: null});
        }
      }, SUCCESS_NOTE_MS);
    }
  };

  const report = (error: CommandError) => notify({text: describeError(error), error: true});

  const probe = async (item: Item): Promise<[ItemId, Probe]> => {
    try {
      return [item.id, {status: 'done', present: await item.probe()}];
    } catch (raw) {
      return [item.id, {status: 'failed', error: toCommandError(raw)}];
    }
  };

  return {
    port: '',
    probes: checking,
    busy: null,
    note: null,
    updateVersion: null,
    popover: false,

    init: async () => {
      const onFocus = () => {
        if (!get().busy) {
          void get().refresh();
        }
      };
      window.addEventListener('focus', onFocus);
      const unlisten = await Promise.all([
        api.onInstallStage(stage => {
          const {busy} = get();
          if (busy) {
            set({busy: {...busy, stage}});
          }
        }),
        api.onUpdate(updateVersion => set({updateVersion})),
        api.onPopover(popover => set({popover})),
      ]);
      const [updateVersion] = await Promise.all([api.getPendingUpdate(), get().refresh()]);
      set({updateVersion});
      return () => {
        window.removeEventListener('focus', onFocus);
        unlisten.forEach(fn => fn());
      };
    },

    refresh: async () => {
      const probes = Object.fromEntries(await Promise.all(ITEMS.map(probe))) as Record<
        ItemId,
        Probe
      >;
      set({probes});
      const failures = ITEMS.flatMap(item => {
        const result = probes[item.id];
        return result.status === 'failed' ? [describeError(result.error, {name: item.name})] : [];
      });
      if (failures.length > 0) {
        notify({text: failures.join('\n'), error: true});
      }
    },

    setPort: port => set({port}),

    run: async (item, action) => {
      const {busy, port} = get();
      const parsed = parsePort(port);
      if (busy || (action.needsPort && parsed === null)) {
        return;
      }
      set({busy: {id: item.id, startedAt: Date.now()}, note: null});
      try {
        const text = await action.run(parsed ?? 0);
        notify({text, error: false});
      } catch (raw) {
        const text = describeError(toCommandError(raw), {name: item.name, port: parsed ?? 0});
        notify({text, error: true});
      } finally {
        await get().refresh();
        set({busy: null});
      }
    },

    dismissNote: () => set({note: null}),

    installUpdate: async () => {
      try {
        await api.installUpdate();
      } catch (raw) {
        report(toCommandError(raw));
      }
    },
  };
});
