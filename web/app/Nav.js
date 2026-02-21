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
              <li>Generate a zero-knowledge proof that you owned Bitcoin on a given date using the <a href="https://github.com/lukechilds/og-zkp" target="_blank" rel="noopener">og-zkp CLI</a></li>
              <li>Submit your proof to the leaderboard</li>
              <li>Verify your identity by posting an attestation on X or Nostr</li>
              <li>Get listed as a verified Bitcoin OG</li>
            </ol>
            <p>Your address and exact date are kept private, only the calendar month and identity are revealed.</p>
          </div>
        </Modal>
      )}

      {open === 'submit' && (
        <Modal title="submit proof" onClose={() => setOpen(null)}>
          <SubmitForm />
        </Modal>
      )}
    </>
  );
}
