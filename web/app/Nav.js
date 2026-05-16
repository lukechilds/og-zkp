'use client';

import { useState } from 'react';
import SubmitForm from './submit/SubmitForm';

function Modal({ onClose, title, children }) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={e => e.stopPropagation()}>
        <button className="modal-close" onClick={onClose}>&times;</button>
        <h2>{title}</h2>
        {children}
      </div>
    </div>
  );
}

export default function Nav() {
  const [open, setOpen] = useState(null);

  return (
    <>
      <div className="header-actions">
        <button className="btn-nav" onClick={() => setOpen('how')}>how it works</button>
        <button className="btn-nav" onClick={() => setOpen('submit')}>submit proof</button>
      </div>

      {open === 'how' && (
        <Modal title="how it works" onClose={() => setOpen(null)}>
          <div className="instructions">
            <ol>
              <li>generate a zero-knowledge proof that you owned bitcoin on a given date using the <a href="https://github.com/lukechilds/og-zkp" target="_blank" rel="noopener">og-zkp CLI</a></li>
              <li>submit your proof to the leaderboard</li>
              <li>verify your (real or throwaway) identity by posting an attestation on X or nostr</li>
              <li>get listed as a verified bitcoin og</li>
            </ol>
            <p>your address and exact date are kept private and never leave your computer. only the calendar month and identity are revealed.</p>
            <a className="btn-primary btn-link" href="https://github.com/lukechilds/og-zkp#readme" target="_blank" rel="noopener">read more</a>
          </div>
        </Modal>
      )}

      {open === 'submit' && (
        <Modal title="submit proof" onClose={() => setOpen(null)}>
          <SubmitForm onSuccess={() => setOpen(null)} />
        </Modal>
      )}
    </>
  );
}
