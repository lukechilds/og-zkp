import { notFound } from 'next/navigation';
import { cache } from 'react';
import { getDb, migrate } from '../../../lib/db';
import ProofClient from './ProofClient';

const getProof = cache(async (id) => {
  const db = getDb();
  await migrate();
  const result = await db.execute({
    sql: "SELECT proof_id, proof, identity, identity_type, block_month, block_inclusion_root, attestation_url, status, created_at, verified_at FROM proofs WHERE proof_id = ?",
    args: [id],
  });
  return result.rows[0] || null;
});

function formatMonth(ts) {
  const n = parseInt(ts, 10);
  if (isNaN(n)) return ts;
  const d = new Date(n * 1000);
  const months = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
  return months[d.getUTCMonth()] + ' ' + d.getUTCFullYear();
}

export async function generateMetadata({ params }) {
  const { id } = await params;
  const proof = await getProof(id);
  if (!proof) return { title: 'proof not found - og-zkp' };

  const month = formatMonth(proof.block_month);
  const status = proof.status === 'verified' ? 'Verified' : 'Pending';
  return {
    title: `${proof.identity} - og-zkp`,
    openGraph: {
      title: `${proof.identity} - og-zkp`,
      description: `${status} Bitcoin OG since ${month}`,
      url: `https://og-zkp.com/proof/${proof.proof_id}`,
    },
  };
}

export const dynamic = 'force-dynamic';

export default async function ProofPage({ params }) {
  const { id } = await params;
  const proof = await getProof(id);
  if (!proof) notFound();

  const month = formatMonth(proof.block_month);
  const isVerified = proof.status === 'verified';

  return (
    <>
      <div className="proof-card">
        <div className="field">
          <div className="field-label">identity</div>
          <div className="field-value">{proof.identity}</div>
        </div>
        <div className="field">
          <div className="field-label">OG status</div>
          <div className="field-value">{month}</div>
        </div>
        <div className="field">
          <div className="field-label">status</div>
          <div className={`field-value ${isVerified ? 'status-verified' : 'status-pending'}`}>
            {isVerified ? 'verified' : 'pending attestation'}
          </div>
        </div>
        <div className="field">
          <div className="field-label">proof ID</div>
          <div className="field-value" style={{ fontSize: '0.75rem', color: '#666' }}>{proof.proof_id}</div>
        </div>
      </div>

      <ProofClient proof={{
        proof_id: proof.proof_id,
        proof: proof.proof,
        identity: proof.identity,
        identity_type: proof.identity_type,
        attestation_url: proof.attestation_url,
        status: proof.status,
      }} />
    </>
  );
}
