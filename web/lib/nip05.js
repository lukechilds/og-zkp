const { decode } = require("nostr-tools/nip19");
const { SimplePool, useWebSocketImplementation } = require("nostr-tools/pool");
const WebSocket = require("ws");

useWebSocketImplementation(WebSocket);

const RELAYS = [
  "wss://relay.damus.io",
  "wss://nos.lol",
  "wss://relay.nostr.band",
];

const NIP05_HTTP_TIMEOUT_MS = 10000;
const RELAY_LOOKUP_TIMEOUT_MS = 10000;
const CRON_RELAY_LOOKUP_TIMEOUT_MS = 30000;

function withTimeout(promise, ms, message) {
  let timeout;
  const timer = new Promise((_, reject) => {
    timeout = setTimeout(() => reject(new Error(message)), ms);
  });
  return Promise.race([promise, timer]).finally(() => clearTimeout(timeout));
}

function npubToHex(npub) {
  const decoded = decode(npub);
  if (decoded.type !== "npub") throw new Error("Not an npub");
  return decoded.data;
}

async function verifyNip05(nip05, hex) {
  try {
    const [name, domain] = nip05.includes("@") ? nip05.split("@") : ["_", nip05];
    const res = await fetch(`https://${domain}/.well-known/nostr.json?name=${encodeURIComponent(name)}`, {
      signal: AbortSignal.timeout(NIP05_HTTP_TIMEOUT_MS),
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
    const event = await withTimeout(
      pool.get(RELAYS, { kinds: [0], authors: [hex], limit: 1 }, { maxWait: RELAY_LOOKUP_TIMEOUT_MS }),
      RELAY_LOOKUP_TIMEOUT_MS + 1000,
      "Timeout resolving NIP-05 metadata"
    );
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

async function getAllNip05s(npubs, timeoutMs = CRON_RELAY_LOOKUP_TIMEOUT_MS) {
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
    const events = await withTimeout(
      pool.querySync(
        RELAYS,
        { kinds: [0], authors: entries.map((e) => e.hex) },
        { maxWait: timeoutMs }
      ),
      timeoutMs + 1000,
      "Timeout fetching NIP-05 metadata"
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
