'use client';

import { useState } from 'react';
import RankModal from './RankModal';

export default function RankInfoButton() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <button
        type="button"
        className="rank-info-trigger"
        aria-label="show rank details"
        aria-haspopup="dialog"
        aria-expanded={open}
        onClick={() => setOpen(true)}
      >
        <span>rank</span>
        <span className="rank-info-icon" aria-hidden="true">i</span>
      </button>
      {open && <RankModal onClose={() => setOpen(false)} />}
    </>
  );
}
