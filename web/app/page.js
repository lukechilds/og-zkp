import Link from 'next/link';
import { getDb, migrate } from '../lib/db';

export const dynamic = 'force-dynamic';

function formatMonth(ts) {
  const n = parseInt(ts, 10);
  if (isNaN(n)) return ts;
  const d = new Date(n * 1000);
  const months = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
  return months[d.getUTCMonth()] + ' ' + d.getUTCFullYear();
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
          <th>identity</th>
          <th>og status</th>
          <th className="type">type</th>
        </tr>
      </thead>
      <tbody>
        {proofs.map((p, i) => (
          <tr key={p.proof_id}>
            <td className="rank">{String(i + 1).padStart(2, '0')}</td>
            <td className="identity"><Link href={`/proof/${p.proof_id}`}>{p.identity}</Link></td>
            <td className="month">{formatMonth(p.block_month)}</td>
            <td className="type">{p.identity_type}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
