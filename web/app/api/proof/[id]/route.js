const { getDb, migrate } = require('../../../../lib/db');

export async function GET(request, { params }) {
  const { id } = await params;

  if (!id) {
    return Response.json({ error: 'Missing proof ID' }, { status: 400 });
  }

  const db = getDb();
  await migrate();

  const result = await db.execute({
    sql: "SELECT proof_id, proof, identity, identity_type, block_month, block_inclusion_root, attestation_url, status, created_at, verified_at FROM proofs WHERE proof_id = ?",
    args: [id],
  });

  if (result.rows.length === 0) {
    return Response.json({ error: 'Proof not found' }, { status: 404 });
  }

  return Response.json(result.rows[0]);
}
