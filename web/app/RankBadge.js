'use client';

import { useState } from 'react';

const RANKS = [
  { name: 'Legend', years: '2009', color: '#e3b341' },
  { name: 'Cypherpunk', years: '2010–2011', color: '#34d399' },
  { name: 'Pioneer', years: '2012–2013', color: '#22d3ee' },
  { name: 'Veteran', years: '2014–2015', color: '#60a5fa' },
  { name: 'Hodler', years: '2016–2017', color: '#a78bfa' },
  { name: 'Stacker', years: '2018–2019', color: '#d29922' },
  { name: 'Pleb', years: '2020+', color: '#818a96' },
];

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

      {open && (
        <div className="modal-overlay" onClick={() => setOpen(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <button className="modal-close" onClick={() => setOpen(false)}>&times;</button>
            <h2>og ranks</h2>
            <p className="rank-modal-desc">Ranks are based on the year you first received Bitcoin.</p>
            <table className="rank-table">
              <thead>
                <tr>
                  <th>rank</th>
                  <th>years</th>
                </tr>
              </thead>
              <tbody>
                {RANKS.map(r => (
                  <tr key={r.name} className={r.name === rank ? 'rank-active' : ''}>
                    <td><span className={`rank-badge rank-${r.name.toLowerCase()}`}>{r.name}</span></td>
                    <td className="rank-years">{r.years}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </>
  );
}
