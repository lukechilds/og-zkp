import { ImageResponse } from 'next/og';
import { getDb, migrate } from '../../../lib/db';
import { formatMonth } from '../../../lib/date';

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
    const shortNpub = `${proof.identity.slice(0, 8)}...${proof.identity.slice(-8)}`;
    displayName = `${nip05} (${shortNpub})`;
  } else {
    displayName = `${proof.identity.slice(0, 8)}...${proof.identity.slice(-8)}`;
  }
  const isVerified = proof.status === 'verified';
  const status = isVerified ? 'Verified' : 'Pending';
  const statusColor = isVerified ? '#22c55e' : '#f59e0b';
  const month = formatMonth(proof.block_month);

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
        <div style={{ display: 'flex', color: '#999999', fontSize: 28 }}>
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
