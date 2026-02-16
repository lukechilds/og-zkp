import './globals.css';
import Nav from './Nav';

export const metadata = {
  title: 'og-zkp',
  description: 'Prove your Bitcoin OG status in zero-knowledge!',
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>
        <div className="container">
          <h1>og-zkp</h1>
          <p className="subtitle">Prove your Bitcoin OG status in zero-knowledge!</p>
          <Nav />
          {children}
          <footer>
            <a href="https://github.com/lukechilds/og-zkp" target="_blank" rel="noopener">Source code</a>
            <span>-</span>
            <a href="https://github.com/lukechilds/og-zkp/issues" target="_blank" rel="noopener">Report a bug</a>
            <span className="credit">A thing by <a href="https://lu.ke" target="_blank" rel="noopener">@lukechilds</a></span>
          </footer>
        </div>
      </body>
    </html>
  );
}
