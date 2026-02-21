import Leaderboard from '../Leaderboard';
import FirstVisit from '../FirstVisit';

export const dynamic = 'force-dynamic';

export default function Home() {
  return <FirstVisit><Leaderboard /></FirstVisit>;
}
