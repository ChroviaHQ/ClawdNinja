import { Network } from 'lucide-react';

import { t } from '../i18n';
import { parsePort, useStore } from '../store';
import { Card, Row } from './ui';

export function EgressCard() {
  const port = useStore(s => s.port);
  const setPort = useStore(s => s.setPort);
  const busy = useStore(s => s.busy !== null);
  const valid = parsePort(port) !== null;
  return (
    <Card>
      <Row
        id='egress'
        icon={Network}
        name={t.egress}
        status={valid ? t.egressHint : t.portRange}
        tone={valid ? undefined : 'bad'}
      >
        <span
          className={`flex h-7 shrink-0 items-center gap-1 rounded-[7px] border bg-surface-2 px-2.5 font-mono text-muted focus-within:border-accent ${
            valid ? 'border-transparent' : 'border-danger'
          }`}
        >
          <span>127.0.0.1</span>
          <span>:</span>
          <input
            aria-label={t.egressPort}
            aria-invalid={!valid}
            disabled={busy}
            inputMode='numeric'
            maxLength={5}
            value={port}
            onChange={event => setPort(event.target.value)}
            style={{width: `${Math.max(port.length, 1)}ch`}}
            className='bg-transparent p-0 text-text outline-none'
          />
        </span>
      </Row>
    </Card>
  );
}
