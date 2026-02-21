import Link from 'next/link';
import { getDb, migrate } from '../lib/db';
import NostrName from './NostrName';

export const dynamic = 'force-dynamic';

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

export default async function Home() {
  const db = getDb();
  await migrate();

  const result = await db.execute(
    "SELECT proof_id, identity, identity_type, block_month FROM proofs WHERE status = 'verified' ORDER BY CAST(block_month AS INTEGER) ASC"
  );
  const proofs = result.rows;

  if (proofs.length === 0) {
    return <p className="empty">no verified proofs yet</p>;
  }

  return (
    <table className="leaderboard">
      <thead>
        <tr>
          <th className="rank">#</th>
          <th className="crown-cell"></th>
          <th>identity</th>
          <th>og status</th>
          <th>age</th>
          <th className="type">type</th>
        </tr>
      </thead>
      <tbody>
        {proofs.map((p, i) => (
          <tr key={p.proof_id}>
            <td className="rank">{String(i + 1).padStart(2, '0')}</td>
            <td className="crown-cell">
              {i === 0 && (
                <svg className="crown" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="var(--green)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M11.562 3.266a.5.5 0 0 1 .876 0L15.39 8.87a1 1 0 0 0 1.516.294L21.183 5.5a.5.5 0 0 1 .798.519l-2.834 10.246a1 1 0 0 1-.956.734H5.81a1 1 0 0 1-.957-.734L2.02 6.02a.5.5 0 0 1 .798-.519l4.276 3.664a1 1 0 0 0 1.516-.294z" />
                  <path d="M5 21h14" />
                </svg>
              )}
            </td>
            <td className="identity">
              <Link href={`/proof/${p.proof_id}`}>
                {p.identity_type === 'nostr'
                  ? <NostrName npub={p.identity} />
                  : p.identity.replace(/^x\.com\//, '@')}
              </Link>
            </td>
            <td className="month">{formatMonth(p.block_month)}</td>
            <td className="age">{formatAge(p.block_month)}</td>
            <td className="type">{p.identity_type}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
