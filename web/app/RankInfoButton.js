'use client';

import { useState } from 'react';
import RankModal from './RankModal';

export default function RankInfoButton() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <button
        type="button"
        className="rank-info-button"
        aria-label="show rank details"
        aria-haspopup="dialog"
        aria-expanded={open}
        onClick={() => setOpen(true)}
      >
        i
      </button>
      {open && <RankModal onClose={() => setOpen(false)} />}
    </>
  );
}
