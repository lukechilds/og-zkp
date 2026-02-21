const { getDb, migrate } = require('../../../lib/db');
const { verifyAttestation } = require('../../../lib/attestation');

export async function POST(request) {
  const body = await request.json();
  const { proof_id, url } = body;

  if (!proof_id || !url) {
    return Response.json({ error: 'Missing proof_id or url' }, { status: 400 });
  }

  const db = getDb();
  await migrate();

  const result = await db.execute({
    sql: 'SELECT * FROM proofs WHERE proof_id = ?',
    args: [proof_id],
  });
  if (result.rows.length === 0) {
    return Response.json({ error: 'Proof not found' }, { status: 404 });
  }

  const proof = result.rows[0];
  if (proof.status === 'verified') {
    return Response.json({ verified: true, message: 'Already verified' });
  }

  let storedUrl;
  try {
    storedUrl = await verifyAttestation(proof, url);
  } catch (e) {
    return Response.json({ error: e.message }, { status: 400 });
  }

  await db.execute({
    sql: "UPDATE proofs SET status = 'verified', attestation_url = ?, verified_at = ? WHERE proof_id = ?",
    args: [storedUrl, Math.floor(Date.now() / 1000), proof_id],
  });

  return Response.json({ verified: true });
}
