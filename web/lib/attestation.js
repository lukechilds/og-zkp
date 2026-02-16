const { schnorr } = require("@noble/curves/secp256k1");
const { bech32 } = require("bech32");
const WebSocket = require("ws");

function getIdentityType(identity) {
  if (identity.startsWith("x.com/")) return "x";
  if (identity.startsWith("npub")) return "nostr";
  return "unknown";
}

// --- X (Twitter) attestation ---

async function verifyXAttestation(proof, url) {
  const match = url.match(
    /(?:x\.com|twitter\.com)\/([^/]+)\/status\/(\d+)/
  );
  if (!match) throw new Error("Invalid X post URL");
  const [, urlUsername, tweetId] = match;

  const identity = proof.identity;
  if (!identity.startsWith("x.com/")) throw new Error("Identity is not an X account");
  const expectedUsername = identity.slice("x.com/".length).toLowerCase();

  const resp = await fetch(
    `https://api.x.com/2/tweets/${tweetId}?expansions=author_id&user.fields=username&tweet.fields=entities`,
    { headers: { Authorization: `Bearer ${process.env.X_BEARER_TOKEN}` } }
  );
  if (!resp.ok) throw new Error(`X API error: ${resp.status}`);
  const data = await resp.json();

  if (data.errors) throw new Error(data.errors[0].message);

  const author = data.includes?.users?.[0];
  if (!author) throw new Error("Could not resolve tweet author");
  if (author.username.toLowerCase() !== expectedUsername) {
    throw new Error("Tweet author does not match proof identity");
  }

  // Twitter replaces URLs with t.co links in text, check expanded_url from entities
  const expectedUrl = `https://og-zkp.com/proof/${proof.proof_id}`;
  const urls = data.data?.entities?.urls || [];
  const hasUrl = urls.some((u) => u.expanded_url === expectedUrl || u.unwound_url === expectedUrl);
  if (!hasUrl) {
    throw new Error("Tweet does not contain proof link");
  }

  // Check the text portion contains the attestation line
  const tweetText = data.data?.text || "";
  const lines = tweetText.split("\n").map((l) => l.trim()).filter(Boolean);
  if (!lines.some((l) => l === "I'm verifying myself on og-zkp")) {
    throw new Error("Tweet does not contain attestation text");
  }

  return true;
}

// --- Nostr attestation ---

function decodeNpub(npub) {
  const { prefix, words } = bech32.decode(npub, 1000);
  if (prefix !== "npub") throw new Error("Not an npub");
  return Buffer.from(bech32.fromWords(words));
}

function decodeNote(note) {
  const { prefix, words } = bech32.decode(note, 1000);
  if (prefix !== "note") throw new Error("Not a note ID");
  return Buffer.from(bech32.fromWords(words));
}

function decodeNevent(nevent) {
  const { prefix, words } = bech32.decode(nevent, 1000);
  if (prefix !== "nevent") throw new Error("Not an nevent");
  const data = Buffer.from(bech32.fromWords(words));
  // TLV format: type (1 byte) + length (1 byte) + value
  // Type 0 = event id (32 bytes), Type 1 = relay URL (string)
  let eventId = null;
  const relays = [];
  let i = 0;
  while (i < data.length) {
    const type = data[i];
    const len = data[i + 1];
    const value = data.slice(i + 2, i + 2 + len);
    if (type === 0 && len === 32) {
      eventId = value;
    } else if (type === 1) {
      relays.push(value.toString("utf8"));
    }
    i += 2 + len;
  }
  if (!eventId) throw new Error("No event ID found in nevent");
  return { eventId, relays };
}

function fetchNostrEvent(eventIdHex, relays) {
  return new Promise((resolve, reject) => {
    let resolved = false;
    const sockets = [];
    const timeout = setTimeout(() => {
      if (!resolved) {
        resolved = true;
        sockets.forEach((ws) => ws.close());
        reject(new Error("Timeout fetching Nostr event"));
      }
    }, 10000);

    for (const relay of relays) {
      try {
        const ws = new WebSocket(relay);
        sockets.push(ws);
        const subId = Math.random().toString(36).slice(2);

        ws.on("open", () => {
          ws.send(JSON.stringify(["REQ", subId, { ids: [eventIdHex] }]));
        });

        ws.on("message", (data) => {
          try {
            const msg = JSON.parse(data.toString());
            if (msg[0] === "EVENT" && msg[1] === subId && msg[2]) {
              if (!resolved) {
                resolved = true;
                clearTimeout(timeout);
                sockets.forEach((s) => s.close());
                resolve(msg[2]);
              }
            }
          } catch {}
        });

        ws.on("error", () => {});
      } catch {}
    }
  });
}

function verifyNostrSignature(event) {
  const serialized = JSON.stringify([
    0,
    event.pubkey,
    event.created_at,
    event.kind,
    event.tags,
    event.content,
  ]);
  const hash = Buffer.from(
    require("crypto").createHash("sha256").update(serialized).digest()
  );
  return schnorr.verify(event.sig, hash, event.pubkey);
}

function validateNostrEvent(event, proof) {
  if (JSON.stringify(event).length > 2048) throw new Error("Nostr event exceeds 2KB limit");

  const identity = proof.identity;
  if (!identity.startsWith("npub")) throw new Error("Identity is not a Nostr npub");

  const expectedPubkey = decodeNpub(identity).toString("hex");

  if (!verifyNostrSignature(event)) {
    throw new Error("Invalid Nostr event signature");
  }

  if (event.pubkey !== expectedPubkey) {
    throw new Error("Event author does not match proof identity");
  }

  const expectedUrl = `https://og-zkp.com/proof/${proof.proof_id}`;
  const lines = event.content.split("\n").map((l) => l.trim()).filter(Boolean);
  if (!lines.some((l) => l === "I'm verifying myself on og-zkp")) {
    throw new Error("Note does not contain attestation text");
  }
  if (!lines.some((l) => l === expectedUrl)) {
    throw new Error("Note does not contain proof link");
  }
}

function tryParseRawEvent(input) {
  try {
    const event = JSON.parse(input);
    if (event.pubkey && event.sig && event.content && typeof event.created_at === "number") {
      return event;
    }
  } catch {}
  return null;
}

async function verifyNostrAttestation(proof, url) {
  // Try raw signed event JSON first
  const rawEvent = tryParseRawEvent(url);
  if (rawEvent) {
    validateNostrEvent(rawEvent, proof);
    return true;
  }

  // Otherwise parse note1/nevent1 reference and fetch from relays
  const neventMatch = url.match(/nevent1[a-z0-9]+/);
  const noteMatch = url.match(/note1[a-z0-9]+/);
  if (!neventMatch && !noteMatch) throw new Error("Invalid Nostr attestation (expected nevent1..., note1..., or raw signed event JSON)");

  let eventIdHex;
  let hintRelays = [];
  if (neventMatch) {
    const decoded = decodeNevent(neventMatch[0]);
    eventIdHex = decoded.eventId.toString("hex");
    hintRelays = decoded.relays;
  } else {
    eventIdHex = decodeNote(noteMatch[0]).toString("hex");
  }

  const defaultRelays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band",
  ];
  const relays = [...new Set([...hintRelays, ...defaultRelays])];

  const event = await fetchNostrEvent(eventIdHex, relays);
  validateNostrEvent(event, proof);

  return true;
}

// --- Dispatcher ---

async function verifyAttestation(proof, url) {
  const identityType = getIdentityType(proof.identity);
  if (identityType === "x") {
    return verifyXAttestation(proof, url);
  } else if (identityType === "nostr") {
    return verifyNostrAttestation(proof, url);
  } else {
    throw new Error("Unsupported identity type");
  }
}

module.exports = { getIdentityType, verifyAttestation };
