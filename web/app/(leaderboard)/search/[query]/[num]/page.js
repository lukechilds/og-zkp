import Leaderboard from '../../../../Leaderboard';

export const dynamic = 'force-dynamic';

export default async function SearchPaginatedPage({ params }) {
  const { query, num } = await params;
  return <Leaderboard query={decodeURIComponent(query)} currentPage={Math.max(1, parseInt(num, 10) || 1)} />;
}
