import { AlertTriangle, CheckCircle2, X } from 'lucide-react';

import { t } from '../i18n';
import { useStore } from '../store';
import { IconButton } from './ui';

export function NoteToast() {
  const note = useStore(s => s.note);
  const dismiss = useStore(s => s.dismissNote);
  if (!note) {
    return null;
  }
  const Icon = note.error ? AlertTriangle : CheckCircle2;
  return (
    <div
      role={note.error ? 'alert' : 'status'}
      className={`toast-in fixed inset-x-4.5 bottom-4.5 z-20 flex items-center gap-2 rounded-[10px] border bg-surface py-1.5 pr-1.5 pl-2.5 text-[11.5px] leading-[1.55] shadow-lg ${
        note.error ? 'border-danger/40' : 'border-line'
      }`}
    >
      <Icon className={`size-3.5 shrink-0 ${note.error ? 'text-danger' : 'text-ok'}`} />
      <span className='flex-1 py-0.5 whitespace-pre-line wrap-anywhere select-text'>
        {note.text}
      </span>
      <IconButton title={t.close} onClick={dismiss}>
        <X className='size-3.5' />
      </IconButton>
    </div>
  );
}
