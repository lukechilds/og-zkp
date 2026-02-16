const { getDb, migrate } = require('../../../lib/db');
const { verify } = require('../../../lib/verify');
const { proofId } = require('../../../lib/crypto');
const { getIdentityType } = require('../../../lib/attestation');

export async function POST(request) {
  const body = await request.json();
  const rawProof = body.proof;

  if (!rawProof || typeof rawProof !== 'string') {
    return Response.json({ error: 'Missing proof string' }, { status: 400 });
  }

  const proof = rawProof.replace(/[\r\n]+/g, '').trim();

  let result;
  try {
    result = await verify(proof);
  } catch (e) {
    return Response.json({ error: e.message }, { status: 400 });
  }

  const id = proofId(proof);
  const identityType = getIdentityType(result.identity);
  if (identityType === 'unknown') {
    return Response.json({ error: 'Unknown identity type' }, { status: 400 });
  }

  const db = getDb();
  await migrate();

  const existing = await db.execute({
    sql: 'SELECT proof_id FROM proofs WHERE proof_id = ?',
    args: [id],
  });
  if (existing.rows.length > 0) {
    return Response.json({ error: 'Proof already submitted', proof_id: id }, { status: 409 });
  }

  await db.execute({
    sql: `INSERT INTO proofs (proof_id, proof, identity, identity_type, block_month, block_inclusion_root, status, created_at)
          VALUES (?, ?, ?, ?, ?, ?, 'pending', ?)`,
    args: [
      id,
      proof,
      result.identity,
      identityType,
      result.block_month,
      result.block_inclusion_root,
      Math.floor(Date.now() / 1000),
    ],
  });

  return Response.json({
    proof_id: id,
    identity: result.identity,
    identity_type: identityType,
    block_month: result.block_month,
  });
}
