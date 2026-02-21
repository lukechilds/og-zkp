import Link from 'next/link';
import { getDb, migrate } from '../lib/db';
import NostrName from './NostrName';

export const dynamic = 'force-dynamic';

const PER_PAGE = 15;

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

export default async function Home({ searchParams }) {
  const { page } = await searchParams;
  const currentPage = Math.max(1, parseInt(page, 10) || 1);
  const offset = (currentPage - 1) * PER_PAGE;

  const db = getDb();
  await migrate();

  const countResult = await db.execute(
    "SELECT COUNT(*) as total FROM proofs WHERE status = 'verified'"
  );
  const total = Number(countResult.rows[0].total);
  const totalPages = Math.ceil(total / PER_PAGE);

  if (total === 0) {
    return <p className="empty">no verified proofs yet</p>;
  }

  const result = await db.execute({
    sql: "SELECT proof_id, identity, identity_type, block_month FROM proofs WHERE status = 'verified' ORDER BY CAST(block_month AS INTEGER) ASC LIMIT ? OFFSET ?",
    args: [PER_PAGE, offset],
  });
  const proofs = result.rows;

  return (
    <>
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
          {proofs.map((p, i) => {
            const rank = offset + i + 1;
            return (
              <tr key={p.proof_id}>
                <td className="rank">{String(rank).padStart(2, '0')}</td>
                <td className="crown-cell">
                  {rank === 1 && (
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
                  <svg className="verified-badge" style={currentPage === 1 ? { animationDelay: `${0.3 + i * 0.12}s` } : { opacity: 1, animation: 'none' }} viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="var(--green)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                    <polyline points="22 4 12 14.01 9 11.01" />
                  </svg>
                </td>
                <td className="month">{formatMonth(p.block_month)}</td>
                <td className="age">{formatAge(p.block_month)}</td>
                <td className="type">{p.identity_type}</td>
              </tr>
            );
          })}
        </tbody>
      </table>

      {totalPages > 1 && (
        <div className="pagination">
          {currentPage > 1 ? (
            <Link href={`/?page=${currentPage - 1}`} className="page-link">&larr; prev</Link>
          ) : (
            <span className="page-link disabled">&larr; prev</span>
          )}
          <span className="page-info">{currentPage} / {totalPages}</span>
          {currentPage < totalPages ? (
            <Link href={`/?page=${currentPage + 1}`} className="page-link">next &rarr;</Link>
          ) : (
            <span className="page-link disabled">next &rarr;</span>
          )}
        </div>
      )}
    </>
  );
}
