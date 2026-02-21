import { ImageResponse } from 'next/og';
import { getDb, migrate } from '../../../lib/db';
import { formatMonth, getRank } from '../../../lib/date';

export const alt = 'og-zkp proof';
export const size = { width: 1200, height: 630 };
export const contentType = 'image/png';

export default async function Image({ params }) {
  const { id } = await params;
  const db = getDb();
  await migrate();
  const result = await db.execute({
    sql: "SELECT identity, identity_type, block_month, status, nip05 FROM proofs WHERE proof_id = ?",
    args: [id],
  });
  const proof = result.rows[0];

  const font = await fetch(
    'https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;700&display=swap'
  ).then(res => res.text());
  const fontUrl = font.match(/src: url\((.+?)\)/)?.[1];
  const fontData = fontUrl ? await fetch(fontUrl).then(res => res.arrayBuffer()) : null;

  if (!proof) {
    return new ImageResponse(
      (
        <div
          style={{
            background: '#0a0a0a',
            width: '100%',
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontFamily: '"JetBrains Mono", monospace',
            color: '#666666',
            fontSize: 36,
          }}
        >
          Proof not found
        </div>
      ),
      {
        ...size,
        fonts: fontData
          ? [{ name: 'JetBrains Mono', data: fontData, style: 'normal', weight: 400 }]
          : [],
      }
    );
  }

  let displayName;
  if (proof.identity_type === 'x') {
    displayName = proof.identity.replace(/^x\.com\//, '@');
  } else if (proof.nip05) {
    const nip05 = proof.nip05.startsWith('_@') ? proof.nip05.slice(2) : proof.nip05;
    const shortNpub = `npub...${proof.identity.slice(-6)}`;
    displayName = `${nip05} (${shortNpub})`;
  } else {
    displayName = `npub...${proof.identity.slice(-6)}`;
  }
  const isVerified = proof.status === 'verified';
  const status = isVerified ? 'Verified' : 'Pending';
  const statusColor = isVerified ? '#22c55e' : '#f59e0b';
  const month = formatMonth(proof.block_month);
  const rank = getRank(proof.block_month);
  const rankColors = {
    Legend: '#e3b341',
    Cypherpunk: '#34d399',
    Pioneer: '#22d3ee',
    Veteran: '#60a5fa',
    Hodler: '#a78bfa',
    Stacker: '#d29922',
    Pleb: '#818a96',
  };
  const rankColor = rankColors[rank] || '#818a96';

  return new ImageResponse(
    (
      <div
        style={{
          background: '#0a0a0a',
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'center',
          padding: '60px 80px',
          fontFamily: '"JetBrains Mono", monospace',
        }}
      >
        <div style={{ display: 'flex', color: '#666666', fontSize: 24, marginBottom: 24 }}>
          og-zkp
        </div>
        <div
          style={{
            display: 'flex',
            color: '#ffffff',
            fontSize: 48,
            fontWeight: 700,
            marginBottom: 32,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
            maxWidth: '100%',
          }}
        >
          {displayName}
        </div>
        <div style={{ display: 'flex', color: statusColor, fontSize: 28, marginBottom: 20 }}>
          {status}
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 16, color: '#999999', fontSize: 28 }}>
          <span style={{ color: rankColor, fontSize: 24, border: `1px solid ${rankColor}40`, borderRadius: 6, padding: '4px 14px', background: `${rankColor}18`, textTransform: 'uppercase', letterSpacing: '0.04em' }}>{rank}</span>
          {`Bitcoin OG since ${month}`}
        </div>
      </div>
    ),
    {
      ...size,
      fonts: fontData
        ? [{ name: 'JetBrains Mono', data: fontData, style: 'normal', weight: 700 }]
        : [],
    }
  );
}
