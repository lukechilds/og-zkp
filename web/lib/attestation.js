const { decode } = require("nostr-tools/nip19");
const { verifyEvent } = require("nostr-tools/pure");
const { SimplePool, useWebSocketImplementation } = require("nostr-tools/pool");
const WebSocket = require("ws");

useWebSocketImplementation(WebSocket);

const X_FETCH_TIMEOUT_MS = 10000;
const NOSTR_FETCH_TIMEOUT_MS = 10000;
const MAX_NOSTR_HINT_RELAYS = 5;

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
    {
      headers: { Authorization: `Bearer ${process.env.X_BEARER_TOKEN}` },
      signal: AbortSignal.timeout(X_FETCH_TIMEOUT_MS),
    }
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
  const decoded = decode(npub);
  if (decoded.type !== "npub") throw new Error("Not an npub");
  return decoded.data;
}

function closeRelayPool(pool) {
  // nostr-tools closes ws connections gracefully, which can keep API requests open.
  for (const relay of pool.relays?.values?.() || []) {
    relay.ws?.terminate?.();
  }
  pool.destroy();
}

async function fetchNostrEvent(eventIdHex, relays, proof) {
  const pool = new SimplePool();
  const controller = new AbortController();
  let sub;
  let timeout;

  return new Promise((resolve, reject) => {
    let settled = false;
    let sawEvent = false;
    let lastValidationError;

    function finish(fn, value) {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      controller.abort();
      sub?.close("attestation lookup complete");
      closeRelayPool(pool);
      fn(value);
    }

    function notFoundError() {
      return sawEvent && lastValidationError
        ? lastValidationError
        : new Error("Nostr event not found on supported relays");
    }

    timeout = setTimeout(() => {
      finish(reject, notFoundError());
    }, NOSTR_FETCH_TIMEOUT_MS + 1000);

    try {
      sub = pool.subscribeEose(relays, { ids: [eventIdHex], limit: 1 }, {
        maxWait: NOSTR_FETCH_TIMEOUT_MS,
        abort: controller.signal,
        onevent(event) {
          sawEvent = true;
          try {
            validateNostrEvent(event, proof);
            finish(resolve, event);
          } catch (e) {
            lastValidationError = e;
          }
        },
        onclose() {
          finish(reject, notFoundError());
        },
      });
    } catch (e) {
      finish(reject, e);
    }
  });
}

function normalizeRelayUrl(relay) {
  if (typeof relay !== "string") return null;

  let value = relay.trim();
  if (!value) return null;
  if (!value.includes("://")) value = `wss://${value}`;

  try {
    const url = new URL(value);
    if (url.protocol === "http:") url.protocol = "ws:";
    if (url.protocol === "https:") url.protocol = "wss:";
    if (url.protocol !== "ws:" && url.protocol !== "wss:") return null;
    url.hash = "";
    url.pathname = url.pathname.replace(/\/+/g, "/");
    if (url.pathname.endsWith("/")) url.pathname = url.pathname.slice(0, -1);
    url.searchParams.sort();
    return url.toString();
  } catch {
    return null;
  }
}

function normalizeRelays(relays) {
  return [...new Set(relays.map(normalizeRelayUrl).filter(Boolean))];
}

function validateNostrEvent(event, proof) {
  if (JSON.stringify(event).length > 2048) throw new Error("Nostr event exceeds 2KB limit");

  const identity = proof.identity;
  if (!identity.startsWith("npub")) throw new Error("Identity is not a Nostr npub");

  const expectedPubkey = decodeNpub(identity);

  if (!verifyEvent(event)) {
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
    return JSON.stringify(rawEvent);
  }

  const match = url.match(/(?:nevent1|note1)[a-z0-9]+/);
  if (!match) throw new Error("Invalid Nostr attestation (expected nevent1..., note1..., or raw signed event JSON)");

  const decoded = decode(match[0]);
  const eventIdHex = decoded.type === "nevent" ? decoded.data.id : decoded.data;
  const hintRelays = decoded.type === "nevent" ? decoded.data.relays || [] : [];

  const relays = normalizeRelays([
    ...hintRelays.slice(0, MAX_NOSTR_HINT_RELAYS),
    ...DEFAULT_RELAYS,
  ]);

  const event = await fetchNostrEvent(eventIdHex, relays, proof);

  return JSON.stringify(event);
}

const DEFAULT_RELAYS = [
  "wss://relay.damus.io",
  "wss://nos.lol",
  "wss://relay.nostr.band",
];

// --- Dispatcher ---

async function verifyAttestation(proof, url) {
  const identityType = getIdentityType(proof.identity);
  if (identityType === "x") {
    await verifyXAttestation(proof, url);
    return url;
  } else if (identityType === "nostr") {
    return await verifyNostrAttestation(proof, url);
  } else {
    throw new Error("Unsupported identity type");
  }
}

module.exports = { getIdentityType, verifyAttestation };
