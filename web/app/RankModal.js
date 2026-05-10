'use client';

const RANKS = [
  { name: 'OG', years: '2009' },
  { name: 'Legend', years: '2010' },
  { name: 'Elder', years: '2011' },
  { name: 'Cypherpunk', years: '2012' },
  { name: 'Pioneer', years: '2013' },
  { name: 'Veteran', years: '2014-2015' },
  { name: 'Hodler', years: '2016-2017' },
  { name: 'Stacker', years: '2018-2019' },
  { name: 'Pleb', years: '2020+' },
];

export default function RankModal({ activeRank, onClose }) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="rank-modal-title"
        onClick={e => e.stopPropagation()}
      >
        <button className="modal-close" onClick={onClose} aria-label="close">&times;</button>
        <h2 id="rank-modal-title">og ranks</h2>
        <p className="rank-modal-desc">ranks are based on the year you first received bitcoin</p>
        <table className="rank-table">
          <thead>
            <tr>
              <th>rank</th>
              <th>years</th>
            </tr>
          </thead>
          <tbody>
            {RANKS.map(r => (
              <tr key={r.name} className={r.name === activeRank ? 'rank-active' : ''}>
                <td><span className={`rank-badge rank-${r.name.toLowerCase()}`}>{r.name}</span></td>
                <td className="rank-years">{r.years}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
