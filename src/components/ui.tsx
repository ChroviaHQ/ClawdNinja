import type { ButtonHTMLAttributes, ComponentType, ReactNode } from 'react';

export type IconComponent = ComponentType<{className?: string;}>;

export type Tone = 'ok' | 'off' | 'wait' | 'bad';

const DOT_TONES: Record<Tone, string> = {
  ok: 'bg-ok',
  off: 'bg-current opacity-50',
  wait: 'blink bg-current opacity-50',
  bad: 'bg-current',
};

export function Card({children}: {children: ReactNode;}) {
  return (
    <ul className='m-0 shrink-0 list-none divide-y divide-line overflow-hidden rounded-[10px] border border-line bg-surface p-0'>
      {children}
    </ul>
  );
}

export function Row(
  {id, icon: Icon, name, status, tone, busy, children}: {
    id: string;
    icon: IconComponent;
    name: string;
    status: string;
    tone?: Tone;
    busy?: boolean;
    children: ReactNode;
  },
) {
  return (
    <li data-item={id} className='relative flex items-center gap-2.5 p-2'>
      <span className='grid size-7 shrink-0 place-items-center rounded-[7px] bg-surface-2'>
        <Icon className='size-3.5' />
      </span>
      <span className='flex min-w-0 flex-1 flex-col gap-px'>
        <b className='truncate leading-4 font-semibold'>{name}</b>
        <small
          className={`flex items-center gap-1.25 text-[11.5px] leading-3.5 whitespace-nowrap ${
            tone === 'bad' ? 'text-danger' : 'text-muted'
          }`}
        >
          {tone && <i className={`size-1.5 shrink-0 rounded-full ${DOT_TONES[tone]}`} />}
          {status}
        </small>
      </span>
      {children}
      {busy && <span className='busy-slide absolute -bottom-px left-0 h-0.5 w-[30%] bg-accent' />}
    </li>
  );
}

const BUTTON_VARIANTS = {
  primary:
    'border-transparent [background:var(--accent-fill,var(--accent))_border-box] text-on-accent hover:enabled:brightness-93',
  secondary: 'border-line bg-surface text-text hover:enabled:bg-surface-2',
};

export function Button(
  {icon: Icon, children, variant = 'secondary', ...props}:
    & ButtonHTMLAttributes<HTMLButtonElement>
    & {icon: IconComponent; variant?: keyof typeof BUTTON_VARIANTS;},
) {
  return (
    <button
      type='button'
      {...props}
      className={`inline-flex h-7 shrink-0 items-center gap-1.25 rounded-[7px] border px-2.5 font-semibold disabled:opacity-45 ${
        BUTTON_VARIANTS[variant]
      }`}
    >
      <Icon className='size-3.5' />
      {children}
    </button>
  );
}

export function IconButton(
  {children, className = '', ...props}: ButtonHTMLAttributes<HTMLButtonElement>,
) {
  return (
    <button
      type='button'
      {...props}
      className={`grid size-7 shrink-0 place-items-center rounded-[7px] text-muted hover:bg-surface-2 ${className}`}
    >
      {children}
    </button>
  );
}
