import { useEffect, useState } from 'react';

import { interpolate, t } from '../i18n';
import { type Item, ITEMS } from '../items';
import { parsePort, type Probe, useStore } from '../store';
import { Button, Card, Row, type Tone } from './ui';

function useElapsedSeconds(startedAt: number | null): number {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    if (startedAt === null) {
      return;
    }
    setNow(Date.now());
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, [startedAt]);
  return startedAt === null ? 0 : Math.max(0, Math.floor((now - startedAt) / 1000));
}

function describeProbe(item: Item, probe: Probe): [string, Tone] {
  switch (probe.status) {
    case 'checking':
      return [t.checking, 'wait'];
    case 'failed':
      return [t.unavailable, 'bad'];
    case 'done':
      return probe.present ? [item.on, 'ok'] : [item.off, 'off'];
  }
}

function ProductRow({item}: {item: Item;}) {
  const probe = useStore(s => s.probes[item.id]);
  const busy = useStore(s => s.busy);
  const portValid = useStore(s => parsePort(s.port) !== null);
  const run = useStore(s => s.run);
  const active = busy?.id === item.id;
  const seconds = useElapsedSeconds(active ? busy.startedAt : null);
  const action = probe.status === 'done' && probe.present ? item.whenOn : item.whenOff;
  const [status, tone] = active
    ? [
      interpolate(t.elapsed, {stage: busy.stage ? t.stages[busy.stage] : t.working, seconds}),
      'wait' as Tone,
    ]
    : describeProbe(item, probe);
  return (
    <Row id={item.id} icon={item.icon} name={item.name} status={status} tone={tone} busy={active}>
      <Button
        icon={action.icon}
        variant={action.primary ? 'primary' : 'secondary'}
        disabled={busy !== null || probe.status !== 'done' || (action.needsPort && !portValid)}
        onClick={() => run(item, action)}
      >
        {action.label}
      </Button>
    </Row>
  );
}

export function ProductCard() {
  return <Card>{ITEMS.map(item => <ProductRow key={item.id} item={item} />)}</Card>;
}
