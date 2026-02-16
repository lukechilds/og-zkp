'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';

export default function Nav() {
  const pathname = usePathname();
  return (
    <nav>
      <Link href="/" className={pathname === '/' ? 'active' : ''}>leaderboard</Link>
      <Link href="/submit" className={pathname === '/submit' ? 'active' : ''}>submit proof</Link>
    </nav>
  );
}
