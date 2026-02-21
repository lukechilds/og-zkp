'use client';

import { useState, useEffect } from 'react';
import { bech32 } from 'bech32';

const RELAYS = [
  'wss://relay.damus.io',
  'wss://relay.nostr.band',
  'wss://nos.lol',
];

const cache = new Map();

function npubToHex(npub) {
  const { words } = bech32.decode(npub, 1000);
  const bytes = bech32.fromWords(words);
  return Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('');
}

export default function NostrName({ npub, nip05, short: showShort = true }) {
  const [name, setName] = useState(nip05 || cache.get(npub) || null);

  useEffect(() => {
    if (nip05 || !npub.startsWith('npub1') || cache.has(npub)) return;

    let resolved = false;
    let hex;
    try { hex = npubToHex(npub); } catch { return; }
    const subId = Math.random().toString(36).slice(2, 10);
    const sockets = [];

    for (const url of RELAYS) {
      try {
        const ws = new WebSocket(url);
        sockets.push(ws);
        ws.onopen = () => {
          ws.send(JSON.stringify(['REQ', subId, { kinds: [0], authors: [hex], limit: 1 }]));
        };
        ws.onmessage = (e) => {
          try {
            const msg = JSON.parse(e.data);
            if (msg[0] === 'EVENT' && msg[2]?.content && !resolved) {
              const meta = JSON.parse(msg[2].content);
              if (meta.nip05) {
                resolved = true;
                const display = meta.nip05.startsWith('_@') ? meta.nip05.slice(2) : meta.nip05;
                cache.set(npub, display);
                setName(display);
                sockets.forEach(s => s.close());
              }
            }
          } catch {}
        };
      } catch {}
    }

    const timeout = setTimeout(() => sockets.forEach(s => s.close()), 5000);
    return () => {
      clearTimeout(timeout);
      sockets.forEach(s => s.close());
    };
  }, [npub]);

  if (!name) {
    if (!showShort) return npub;
    return `npub...${npub.slice(-6)}`;
  }
  if (!showShort) return name;
  return name;
}
