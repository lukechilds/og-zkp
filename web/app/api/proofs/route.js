const { getDb, migrate } = require('../../../lib/db');

export async function GET() {
  const db = getDb();
  await migrate();

  const result = await db.execute(
    "SELECT proof_id, identity, identity_type, block_month FROM proofs WHERE status = 'verified' ORDER BY CAST(block_month AS INTEGER) ASC"
  );

  return Response.json(result.rows);
}
