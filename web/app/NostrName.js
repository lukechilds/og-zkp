'use client';

import { useState, useEffect } from 'react';
import { decode } from 'nostr-tools/nip19';
import { SimplePool } from 'nostr-tools/pool';

const RELAYS = [
  'wss://relay.damus.io',
  'wss://relay.nostr.band',
  'wss://nos.lol',
];

const cache = new Map();

function npubToHex(npub) {
  const decoded = decode(npub);
  if (decoded.type !== 'npub') throw new Error('not an npub');
  return decoded.data;
}

export default function NostrName({ npub, nip05, short: showShort = true }) {
  const [name, setName] = useState(nip05 || cache.get(npub) || null);

  useEffect(() => {
    if (nip05 || !npub.startsWith('npub1') || cache.has(npub)) return;

    let cancelled = false;
    let hex;
    try { hex = npubToHex(npub); } catch { return; }
    const pool = new SimplePool();

    pool.get(RELAYS, { kinds: [0], authors: [hex], limit: 1 }, { maxWait: 5000 })
      .then((event) => {
        if (cancelled || !event?.content) return;
        const meta = JSON.parse(event.content);
        if (!meta.nip05) return;
        const display = meta.nip05.startsWith('_@') ? meta.nip05.slice(2) : meta.nip05;
        cache.set(npub, display);
        setName(display);
      })
      .catch(() => {})
      .finally(() => pool.close(RELAYS));

    return () => {
      cancelled = true;
      pool.close(RELAYS);
    };
  }, [npub]);

  if (!name) {
    if (!showShort) return npub;
    return `${npub.slice(0, 8)}...${npub.slice(-6)}`;
  }
  if (!showShort) return name;
  const atIndex = name.indexOf('@');
  if (atIndex === -1) return name;
  return <>{name.slice(0, atIndex)}<span className="nip05-domain">@{name.slice(atIndex + 1)}</span></>;
}
