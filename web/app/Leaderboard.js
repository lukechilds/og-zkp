import Link from 'next/link';
import { getDb, migrate } from '../lib/db';
import { formatMonth, formatAge, getRank } from '../lib/date';
import NostrName from './NostrName';
import RankBadge from './RankBadge';
import RankInfoButton from './RankInfoButton';

const PER_PAGE = 15;

export default async function Leaderboard({ query = '', currentPage = 1 }) {
  const offset = (currentPage - 1) * PER_PAGE;

  const db = getDb();
  await migrate();

  const whereArgs = query ? [`%${query}%`, `%${query}%`] : [];

  const dedup = `SELECT proof_id, identity, identity_type, block_month, nip05
    FROM proofs p1
    WHERE status = 'verified'
    AND CAST(block_month AS INTEGER) = (
      SELECT MIN(CAST(block_month AS INTEGER))
      FROM proofs p2
      WHERE p2.identity = p1.identity AND p2.status = 'verified'
    )
    AND p1.rowid = (
      SELECT MIN(p3.rowid) FROM proofs p3
      WHERE p3.identity = p1.identity AND p3.status = 'verified'
      AND CAST(p3.block_month AS INTEGER) = CAST(p1.block_month AS INTEGER)
    )`;

  const countResult = await db.execute({
    sql: query
      ? `SELECT COUNT(*) as total FROM (${dedup}) WHERE identity LIKE ? OR nip05 LIKE ?`
      : `SELECT COUNT(*) as total FROM (${dedup})`,
    args: whereArgs,
  });
  const total = Number(countResult.rows[0].total);
  const totalPages = Math.ceil(total / PER_PAGE);

  const result = await db.execute({
    sql: `SELECT * FROM (
      SELECT *, ROW_NUMBER() OVER (ORDER BY CAST(block_month AS INTEGER) ASC) as rank
      FROM (${dedup})
    ) WHERE ${query ? '(identity LIKE ? OR nip05 LIKE ?)' : '1=1'} LIMIT ? OFFSET ?`,
    args: [...whereArgs, PER_PAGE, offset],
  });
  const proofs = result.rows;

  const pageHref = (p) => {
    if (query) return p === 1 ? `/search/${encodeURIComponent(query)}` : `/search/${encodeURIComponent(query)}/${p}`;
    return p === 1 ? '/' : `/page/${p}`;
  };

  return (
    <>
      <h2 className="leaderboard-title">leaderboard</h2>
      {total === 0 ? (
        <p className="empty">{query ? 'no results' : 'no verified proofs yet'}</p>
      ) : (
        <table className="leaderboard">
          <thead>
            <tr>
              <th className="rank">#</th>
              <th className="crown-cell"></th>
              <th>identity</th>
              <th><span className="rank-heading">rank <RankInfoButton /></span></th>
              <th>bitcoiner since</th>
              <th>age</th>
            </tr>
          </thead>
          <tbody>
            {proofs.map((p, i) => {
              const rank = Number(p.rank);
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
                        ? <NostrName npub={p.identity} nip05={p.nip05} />
                        : p.identity.replace(/^x\.com\//, '@')}
                    </Link>
                    <span className="identity-type">{p.identity_type === 'x' ? 'x.com' : p.identity_type}</span>
                    <svg className="verified-badge" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="var(--green)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                      <polyline points="22 4 12 14.01 9 11.01" />
                    </svg>
                  </td>
                  <td className="tier"><RankBadge rank={getRank(p.block_month)} /></td>
                  <td className="month">{formatMonth(p.block_month)}</td>
                  <td className="age">{formatAge(p.block_month)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}

      {totalPages > 1 && (
        <div className="pagination">
          {currentPage > 1 ? (
            <Link href={pageHref(currentPage - 1)} className="page-link">&larr; prev</Link>
          ) : (
            <span className="page-link disabled">&larr; prev</span>
          )}
          <span className="page-info">{currentPage} / {totalPages}</span>
          {currentPage < totalPages ? (
            <Link href={pageHref(currentPage + 1)} className="page-link">next &rarr;</Link>
          ) : (
            <span className="page-link disabled">next &rarr;</span>
          )}
        </div>
      )}
    </>
  );
}
