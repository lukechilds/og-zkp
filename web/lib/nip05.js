const { bech32 } = require("bech32");
const { SimplePool, useWebSocketImplementation } = require("nostr-tools/pool");
const WebSocket = require("ws");

useWebSocketImplementation(WebSocket);

const RELAYS = [
  "wss://relay.damus.io",
  "wss://nos.lol",
  "wss://relay.nostr.band",
];

function npubToHex(npub) {
  const { prefix, words } = bech32.decode(npub, 1000);
  if (prefix !== "npub") throw new Error("Not an npub");
  return Buffer.from(bech32.fromWords(words)).toString("hex");
}

async function verifyNip05(nip05, hex) {
  try {
    const [name, domain] = nip05.includes("@") ? nip05.split("@") : ["_", nip05];
    const res = await fetch(`https://${domain}/.well-known/nostr.json?name=${encodeURIComponent(name)}`, {
      signal: AbortSignal.timeout(5000),
    });
    if (!res.ok) return false;
    const data = await res.json();
    return data.names?.[name] === hex;
  } catch {
    return false;
  }
}

async function resolveNip05(npub) {
  let hex;
  try {
    hex = npubToHex(npub);
  } catch {
    return null;
  }

  const pool = new SimplePool();
  try {
    const event = await pool.get(RELAYS, { kinds: [0], authors: [hex], limit: 1 });
    if (!event?.content) return null;

    const meta = JSON.parse(event.content);
    if (!meta.nip05) return null;

    const verified = await verifyNip05(meta.nip05, hex);
    if (!verified) return null;

    return meta.nip05.startsWith("_@") ? meta.nip05.slice(2) : meta.nip05;
  } catch {
    return null;
  } finally {
    pool.close(RELAYS);
  }
}

async function getAllNip05s(npubs) {
  const entries = [];
  for (const npub of npubs) {
    try {
      entries.push({ npub, hex: npubToHex(npub) });
    } catch {}
  }
  if (entries.length === 0) return new Map();

  const pool = new SimplePool();
  const results = new Map();

  try {
    const events = await pool.querySync(
      RELAYS,
      { kinds: [0], authors: entries.map((e) => e.hex) }
    );

    const hexToNpub = new Map(entries.map((e) => [e.hex, e.npub]));

    for (const event of events) {
      const npub = hexToNpub.get(event.pubkey);
      if (!npub || results.has(npub)) continue;
      try {
        const meta = JSON.parse(event.content);
        if (meta.nip05) {
          const display = meta.nip05.startsWith("_@") ? meta.nip05.slice(2) : meta.nip05;
          results.set(npub, { nip05: display, raw: meta.nip05, hex: event.pubkey });
        }
      } catch {}
    }
  } finally {
    pool.close(RELAYS);
  }

  return results;
}

module.exports = { resolveNip05, verifyNip05, getAllNip05s };
