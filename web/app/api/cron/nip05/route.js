const { getDb, migrate } = require('../../../../lib/db');
const { resolveNip05 } = require('../../../../lib/nip05');

export async function GET(request) {
  const authHeader = request.headers.get('authorization');
  if (authHeader !== `Bearer ${process.env.CRON_SECRET}`) {
    return Response.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const db = getDb();
  await migrate();

  const result = await db.execute(
    "SELECT proof_id, identity FROM proofs WHERE identity_type = 'nostr'"
  );

  let updated = 0;
  for (const row of result.rows) {
    const nip05 = await resolveNip05(row.identity);
    if (nip05) {
      await db.execute({
        sql: 'UPDATE proofs SET nip05 = ? WHERE proof_id = ?',
        args: [nip05, row.proof_id],
      });
      updated++;
    }
  }

  return Response.json({ total: result.rows.length, updated });
}
