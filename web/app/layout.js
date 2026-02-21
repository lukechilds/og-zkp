import { JetBrains_Mono } from 'next/font/google';
import './globals.css';
import Nav from './Nav';

const mono = JetBrains_Mono({ subsets: ['latin'] });

export const metadata = {
  title: 'og-zkp',
  description: 'Prove your Bitcoin OG status in zero-knowledge!',
};

export default function RootLayout({ children }) {
  return (
    <html lang="en" className={mono.className}>
      <body>
        <div className="site">
          <header>
            <h1>og-zkp</h1>
            <p className="tagline">Prove your Bitcoin OG status in zero-knowledge!</p>
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
