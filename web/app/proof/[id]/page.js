import { notFound } from 'next/navigation';
import { cache } from 'react';
import { getDb, migrate } from '../../../lib/db';
import ProofClient from './ProofClient';
import NostrName from '../../NostrName';

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

function formatAge(ts) {
  const n = parseInt(ts, 10);
  if (isNaN(n)) return '';
  const then = new Date(n * 1000);
  const now = new Date();
  let years = now.getUTCFullYear() - then.getUTCFullYear();
  let months = now.getUTCMonth() - then.getUTCMonth();
  if (months < 0) { years--; months += 12; }
  if (years > 0 && months > 0) return `${years}y ${months}m`;
  if (years > 0) return `${years}y`;
  if (months > 0) return `${months}m`;
  return '<1m';
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
  const age = formatAge(proof.block_month);
  const isVerified = proof.status === 'verified';

  return (
    <>
      <div className="proof-card">
        <div className="field">
          <div className="field-label">identity</div>
          <div className="field-value">
            {proof.identity_type === 'x' ? proof.identity.replace(/^x\.com\//, '@') : proof.identity}
          </div>
          {proof.identity_type === 'nostr' && (
            <div className="field-sub">
              <span className="field-sub-label">nip-05</span> <NostrName npub={proof.identity} short={false} />
            </div>
          )}
        </div>
        <div className="field">
          <div className="field-label">og status</div>
          <div className="field-value">{month}</div>
        </div>
        <div className="field">
          <div className="field-label">age</div>
          <div className="field-value">{age}</div>
        </div>
        <div className="field">
          <div className="field-label">verification</div>
          <div className={`field-value ${isVerified ? 'status-verified' : 'status-pending'}`}>
            {isVerified ? '✓ verified' : 'pending attestation'}
          </div>
        </div>
        <div className="field">
          <div className="field-label">proof id</div>
          <div className="field-value field-muted">{proof.proof_id}</div>
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
