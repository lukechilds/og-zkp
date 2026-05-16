import { notFound } from 'next/navigation';
import { cache } from 'react';
import { getDb, migrate } from '../../../lib/db';
import { formatMonth, formatAge, getRank } from '../../../lib/date';
import ProofClient from './ProofClient';
import NostrName from '../../NostrName';
import RankBadge from '../../RankBadge';

const getProof = cache(async (id) => {
  const db = getDb();
  await migrate();
  const result = await db.execute({
    sql: "SELECT proof_id, proof, identity, identity_type, block_month, block_inclusion_root, attestation_url, status, created_at, verified_at, nip05 FROM proofs WHERE proof_id = ?",
    args: [id],
  });
  return result.rows[0] || null;
});

export async function generateMetadata({ params }) {
  const { id } = await params;
  const proof = await getProof(id);
  if (!proof) return { title: 'proof not found - og-zkp' };

  const month = formatMonth(proof.block_month);
  const rank = getRank(proof.block_month);
  const status = proof.status === 'verified' ? 'verified' : 'pending';
  const displayName = proof.nip05 || proof.identity;
  return {
    title: `${displayName} - og-zkp`,
    openGraph: {
      title: `${displayName} - og-zkp`,
      description: `${status} — ${rank.toLowerCase()} — bitcoiner since ${month}`,
      url: `https://og-zkp.com/proof/${proof.proof_id}`,
    },
    twitter: {
      card: 'summary_large_image',
    },
  };
}

export const dynamic = 'force-dynamic';

export default async function ProofPage({ params }) {
  const { id } = await params;
  const proof = await getProof(id);
  if (!proof) notFound();

  const month = formatMonth(proof.block_month);
  const rank = getRank(proof.block_month);
  const age = formatAge(proof.block_month);
  const isVerified = proof.status === 'verified';

  return (
    <>
      <div className="proof-card">
        <table className="proof-table">
          <tbody>
            <tr>
              <td className="proof-label">identity</td>
              <td className="proof-value">
                {proof.identity_type === 'x'
                  ? <a href={`https://${proof.identity}`} target="_blank" rel="noopener">{proof.identity.replace(/^x\.com\//, '@')}</a>
                  : <a href={`https://njump.me/${proof.identity}`} target="_blank" rel="noopener">{proof.identity}</a>
                }
                {proof.identity_type === 'nostr' && proof.nip05 && (
                  <div className="field-sub">
                    <span className="field-sub-label">nip-05</span> <NostrName npub={proof.identity} nip05={proof.nip05} short={false} />
                  </div>
                )}
              </td>
            </tr>
            <tr className="proof-rank-row">
              <td className="proof-label">rank</td>
              <td className="proof-value"><RankBadge rank={rank} /></td>
            </tr>
            <tr>
              <td className="proof-label">bitcoiner since</td>
              <td className="proof-value">
                <span className="proof-mobile-rank"><RankBadge rank={rank} /><span className="proof-mobile-since"> since </span></span>
                {month}
                <span className="proof-mobile-age"> - {age}</span>
              </td>
            </tr>
            <tr className="proof-age-row">
              <td className="proof-label">age</td>
              <td className="proof-value">{age}</td>
            </tr>
            <tr>
              <td className="proof-label">verification</td>
              <td className={`proof-value ${isVerified ? 'status-verified' : 'status-pending'}`}>
                {isVerified ? '✓ verified' : 'pending attestation'}
              </td>
            </tr>
          </tbody>
        </table>
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
