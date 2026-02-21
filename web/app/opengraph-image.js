import { ImageResponse } from 'next/og';

export const alt = 'og-zkp - Prove your Bitcoin OG status in zero-knowledge';
export const size = { width: 1200, height: 630 };
export const contentType = 'image/png';

export default async function Image() {
  const font = await fetch(
    'https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@700&display=swap'
  ).then(res => res.text());
  const fontUrl = font.match(/src: url\((.+?)\)/)?.[1];
  const fontData = fontUrl ? await fetch(fontUrl).then(res => res.arrayBuffer()) : null;

  return new ImageResponse(
    (
      <div
        style={{
          background: '#0a0a0a',
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          fontFamily: '"JetBrains Mono", monospace',
        }}
      >
        <div style={{ color: '#ffffff', fontSize: 72, fontWeight: 700 }}>
          og-zkp
        </div>
        <div style={{ color: '#666666', fontSize: 28, marginTop: 24 }}>
          Prove your Bitcoin OG status in zero-knowledge
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
