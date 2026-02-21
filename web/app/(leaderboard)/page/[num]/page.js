import Leaderboard from '../../../Leaderboard';

export const dynamic = 'force-dynamic';

export default async function PaginatedHome({ params }) {
  const { num } = await params;
  return <Leaderboard currentPage={Math.max(1, parseInt(num, 10) || 1)} />;
}
