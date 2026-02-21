const { getDb, migrate } = require('../../../../lib/db');
const { getAllNip05s, verifyNip05 } = require('../../../../lib/nip05');

export async function GET(request) {
  const authHeader = request.headers.get('authorization');
  if (authHeader !== `Bearer ${process.env.CRON_SECRET}`) {
    return Response.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const db = getDb();
  await migrate();

  const result = await db.execute(
    "SELECT proof_id, identity, nip05 FROM proofs WHERE identity_type = 'nostr'"
  );

  const rows = result.rows;
  const resolved = await getAllNip05s(rows.map((r) => r.identity));

  let updated = 0;

  await Promise.all(rows.map(async (row) => {
    const entry = resolved.get(row.identity);
    if (!entry) return;
    if (entry.nip05 === row.nip05) return;

    const verified = await verifyNip05(entry.raw, entry.hex);
    if (!verified) return;

    await db.execute({
      sql: 'UPDATE proofs SET nip05 = ? WHERE proof_id = ?',
      args: [entry.nip05, row.proof_id],
    });
    updated++;
  }));

  return Response.json({ total: rows.length, updated });
}
