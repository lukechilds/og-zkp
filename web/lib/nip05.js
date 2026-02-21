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

function resolveNip05(npub, timeoutMs = 3000) {
  return new Promise((resolve) => {
    let done = false;
    let hex;
    try {
      hex = npubToHex(npub);
    } catch {
      return resolve(null);
    }

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
                const display = meta.nip05.startsWith("_@")
                  ? meta.nip05.slice(2)
                  : meta.nip05;
                resolve(display);
              }
            }
          } catch {}
        });

        ws.on("error", () => {});
      } catch {}
    }
  });
}

module.exports = { resolveNip05 };
