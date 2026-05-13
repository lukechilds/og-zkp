'use client';

import { useEffect, useState } from 'react';
import { createPortal } from 'react-dom';
import RankModal from './RankModal';

export default function RankBadge({ rank }) {
  const [open, setOpen] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  return (
    <>
      <span
        className={`rank-badge rank-${rank.toLowerCase()}`}
        onClick={() => setOpen(true)}
      >
        {rank}
      </span>

      {open && mounted && createPortal(
        <RankModal activeRank={rank} onClose={() => setOpen(false)} />,
        document.body
      )}
    </>
  );
}
