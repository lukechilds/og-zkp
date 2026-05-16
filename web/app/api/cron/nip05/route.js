const { getDb, migrate } = require('../../../../lib/db');
const { getAllNip05s, verifyNip05 } = require('../../../../lib/nip05');

export async function GET(request) {
  const authHeader = request.headers.get('authorization');
  if (authHeader !== `Bearer ${process.env.CRON_SECRET}`) {
    return Response.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const db = getDb();
  await migrate();

  const result = await db.execute(`
    SELECT identity, MAX(nip05) as nip05, MIN(COALESCE(nip05_checked_at, 0)) as oldest_checked_at
    FROM proofs
    WHERE identity_type = 'nostr' AND status = 'verified'
    GROUP BY identity
    ORDER BY oldest_checked_at ASC, MIN(COALESCE(verified_at, created_at)) ASC, identity ASC
    LIMIT 100
  `);

  const rows = result.rows;
  const resolved = await getAllNip05s(rows.map((r) => r.identity));
  const checkedAt = Math.floor(Date.now() / 1000);

  let checked = 0;
  let updated = 0;

  await Promise.all(rows.map(async (row) => {
    const entry = resolved.get(row.identity);

    if (entry) {
      const verified = await verifyNip05(entry.raw, entry.hex);
      if (verified) {
        await db.execute({
          sql: `UPDATE proofs
                SET nip05 = ?, nip05_checked_at = ?
                WHERE identity = ? AND identity_type = 'nostr' AND status = 'verified'`,
          args: [entry.nip05, checkedAt, row.identity],
        });
        if (entry.nip05 !== row.nip05) updated++;
        checked++;
        return;
      }
    }

    await db.execute({
      sql: `UPDATE proofs
            SET nip05_checked_at = ?
            WHERE identity = ? AND identity_type = 'nostr' AND status = 'verified'`,
      args: [checkedAt, row.identity],
    });
    checked++;
  }));

  return Response.json({ checked, selected: rows.length, updated });
}
