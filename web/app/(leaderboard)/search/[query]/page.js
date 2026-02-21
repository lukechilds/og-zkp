import Leaderboard from '../../../Leaderboard';

export const dynamic = 'force-dynamic';

export default async function SearchPage({ params }) {
  const { query } = await params;
  return <Leaderboard query={decodeURIComponent(query)} />;
}
