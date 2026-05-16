const { createClient } = require("@libsql/client");

let db;

function getDb() {
  if (!db) {
    db = createClient({
      url: process.env.TURSO_URL,
      authToken: process.env.TURSO_AUTH_TOKEN,
    });
  }
  return db;
}

async function migrate() {
  const db = getDb();
  await db.execute(`
    CREATE TABLE IF NOT EXISTS proofs (
      proof_id TEXT PRIMARY KEY,
      proof TEXT NOT NULL,
      identity TEXT NOT NULL,
      identity_type TEXT NOT NULL,
      block_month TEXT NOT NULL,
      block_inclusion_root TEXT NOT NULL,
      attestation_url TEXT,
      status TEXT NOT NULL DEFAULT 'pending',
      created_at INTEGER NOT NULL,
      verified_at INTEGER
    )
  `);
  try {
    await db.execute(`ALTER TABLE proofs ADD COLUMN nip05 TEXT`);
  } catch {}
  try {
    await db.execute(`ALTER TABLE proofs ADD COLUMN nip05_checked_at INTEGER`);
  } catch {}
  await db.execute(`
    CREATE INDEX IF NOT EXISTS idx_proofs_nip05_queue
    ON proofs (identity_type, status, nip05_checked_at)
  `);
}

module.exports = { getDb, migrate };
