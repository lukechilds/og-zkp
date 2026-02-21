import { JetBrains_Mono } from 'next/font/google';
import './globals.css';
import Nav from './Nav';

const mono = JetBrains_Mono({ subsets: ['latin'] });

export const metadata = {
  title: 'og-zkp',
  description: 'Prove your Bitcoin OG status in zero-knowledge',
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
            <div>
              <h1><a href="/">og-zkp</a></h1>
              <p className="tagline">Prove your Bitcoin OG status in zero-knowledge</p>
            </div>
            <Nav />
          </header>
          <main>{children}</main>
          <footer>
            <a href="https://github.com/lukechilds/og-zkp" target="_blank" rel="noopener">Source code</a>
            <span className="sep">-</span>
            <a href="https://github.com/lukechilds/og-zkp/issues" target="_blank" rel="noopener">Report a bug</a>
            <span className="credit">A thing by <a href="https://lu.ke" target="_blank" rel="noopener">@lukechilds</a></span>
          </footer>
        </div>
      </body>
    </html>
  );
}
