'use client';

import { useState } from 'react';
import RankModal from './RankModal';

export default function RankBadge({ rank }) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <span
        className={`rank-badge rank-${rank.toLowerCase()}`}
        onClick={() => setOpen(true)}
      >
        {rank}
      </span>

      {open && <RankModal activeRank={rank} onClose={() => setOpen(false)} />}
    </>
  );
}
