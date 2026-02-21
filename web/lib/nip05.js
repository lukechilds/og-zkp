const { bech32 } = require("bech32");
const WebSocket = require("ws");

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

function fetchNip05FromRelays(hex, timeoutMs = 3000) {
  return new Promise((resolve) => {
    let done = false;
    const sockets = [];
    const subId = Math.random().toString(36).slice(2, 10);

    const timeout = setTimeout(() => {
      if (!done) {
        done = true;
        sockets.forEach((ws) => ws.close());
        resolve(null);
      }
    }, timeoutMs);

    for (const relay of RELAYS) {
      try {
        const ws = new WebSocket(relay);
        sockets.push(ws);

        ws.on("open", () => {
          ws.send(
            JSON.stringify(["REQ", subId, { kinds: [0], authors: [hex], limit: 1 }])
          );
        });

        ws.on("message", (data) => {
          try {
            const msg = JSON.parse(data.toString());
            if (msg[0] === "EVENT" && msg[2]?.content && !done) {
              const meta = JSON.parse(msg[2].content);
              if (meta.nip05) {
                done = true;
                clearTimeout(timeout);
                sockets.forEach((s) => s.close());
                resolve(meta.nip05);
              }
            }
          } catch {}
        });

        ws.on("error", () => {});
      } catch {}
    }
  });
}

async function resolveNip05(npub, timeoutMs = 3000) {
  let hex;
  try {
    hex = npubToHex(npub);
  } catch {
    return null;
  }

  const nip05 = await fetchNip05FromRelays(hex, timeoutMs);
  if (!nip05) return null;

  const verified = await verifyNip05(nip05, hex);
  if (!verified) return null;

  return nip05.startsWith("_@") ? nip05.slice(2) : nip05;
}

module.exports = { resolveNip05 };
