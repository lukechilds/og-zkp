import { JetBrains_Mono } from 'next/font/google';
import { createHash } from 'crypto';
import './globals.css';
import Nav from './Nav';

const NUM_LINES = 5;
const MIDDLE = 2;
const TAGLINE_LINE = NUM_LINES - 1;

function sha256(input) {
  return createHash('sha256').update(input).digest('hex');
}

function generateHashes() {
  const lines = [];
  let hash = sha256('luke');
  for (let i = 0; i < NUM_LINES; i++) {
    lines.push(hash);
    hash = sha256(hash);
  }
  return lines;
}

const hashes = generateHashes();
const title = ' og-zkp ';
const start = Math.floor((64 - title.length) / 2);
const tagline = ' prove your bitcoin og status in zero-knowledge ';
const taglineStart = Math.floor((64 - tagline.length) / 2);

const mono = JetBrains_Mono({ subsets: ['latin'] });

export const metadata = {
  title: 'og-zkp',
  description: 'prove your bitcoin og status in zero-knowledge',
  openGraph: {
    siteName: 'og-zkp',
    url: 'https://og-zkp.com',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
  },
};

export default function RootLayout({ children }) {
  return (
    <html lang="en" className={mono.className}>
      <body>
        <div className="site">
          <header>
            <a href="/" className="header-link" aria-label="og-zkp home">
              <div className="hash-header">
                {hashes.map((hash, i) => (
                  i === MIDDLE ? (
                    <div key={hash}>
                      <span className="hash-dim">{hash.slice(0, start)}</span>
                      <span className="hash-text">{title}</span>
                      <span className="hash-dim">{hash.slice(start + title.length)}</span>
                    </div>
                  ) : i === TAGLINE_LINE ? (
                    <div key={hash}>
                      <span className="hash-dim">{hash.slice(0, taglineStart)}</span>
                      <span className="hash-tagline">{tagline}</span>
                      <span className="hash-dim">{hash.slice(taglineStart + tagline.length)}</span>
                    </div>
                  ) : (
                    <div key={hash} className="hash-dim">{hash}</div>
                  )
                ))}
              </div>
            </a>
            <Nav />
          </header>
          <main>{children}</main>
          <footer>
            <a href="https://github.com/lukechilds/og-zkp" target="_blank" rel="noopener">source code</a>
            <span className="credit">a thing by <a href="https://lu.ke" target="_blank" rel="noopener">@lukechilds</a></span>
          </footer>
        </div>
      </body>
    </html>
  );
}
