'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';

function CopyButton({ text }) {
  const [label, setLabel] = useState('copy');

  function handleCopy() {
    navigator.clipboard.writeText(text).then(() => {
      setLabel('copied');
      setTimeout(() => setLabel('copy'), 1500);
    });
  }

  return (
    <button className="copy-btn" onClick={handleCopy}>{label}</button>
  );
}

export default function ProofClient({ proof }) {
  const router = useRouter();
  const isVerified = proof.status === 'verified';
  const pageUrl = `https://og-zkp.com/proof/${proof.proof_id}`;
  const attestString = `I'm verifying myself on og-zkp\n\n${pageUrl}`;
  const isX = proof.identity_type === 'x';

  return (
    <>
      {isVerified && proof.attestation_url && (
        <AttestationDisplay url={proof.attestation_url} />
      )}

      {!isVerified && (
        <AttestationForm
          proofId={proof.proof_id}
          identity={proof.identity}
          isX={isX}
          attestString={attestString}
          pageUrl={pageUrl}
          onVerified={() => router.refresh()}
        />
      )}

      <div className="proof-section">
        <h2>proof</h2>
        <div className="proof-string-wrap">
          <pre className="proof-string">{proof.proof}</pre>
          <CopyButton text={proof.proof} />
        </div>
      </div>
    </>
  );
}

function AttestationDisplay({ url }) {
  const isXAttestation = url.match(/(?:x\.com|twitter\.com)\/[^/]+\/status\/(\d+)/);

  if (isXAttestation) {
    return (
      <div className="attestation-section">
        <h2>attestation</h2>
        <div className="tweet-embed"></div>
        <div className="attestation-link">
          <a href={url} target="_blank" rel="noopener">{url}</a>
        </div>
      </div>
    );
  }

  return (
    <div className="attestation-section">
      <h2>attestation</h2>
      <div className="nostr-embed">
        <pre className="attest-text">{url}</pre>
      </div>
    </div>
  );
}

function AttestationForm({ proofId, identity, isX, attestString, pageUrl, onVerified }) {
  const [url, setUrl] = useState('');
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(e) {
    e.preventDefault();
    setError('');
    setSuccess('');
    setSubmitting(true);

    const trimmed = url.trim();
    try {
      const res = await fetch('/api/attest', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ proof_id: proofId, url: trimmed }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data.error || 'attestation failed');
      setSuccess('attestation verified!');
      setTimeout(() => onVerified(), 1500);
    } catch (err) {
      setError(err.message);
      setSubmitting(false);
    }
  }

  return (
    <div className="attestation-section">
      <h2>complete attestation</h2>
      <div className="instructions">
        {isX ? (
          <ol>
            <li>
              Post a tweet from <strong>{identity}</strong> with this exact text:
              <div className="attest-block">
                <pre className="attest-text">{attestString}</pre>
                <CopyButton text={attestString} />
              </div>
            </li>
            <li>Paste the tweet URL below</li>
          </ol>
        ) : (
          <ol>
            <li>
              Sign a Nostr note from your npub with this exact text:
              <div className="attest-block">
                <pre className="attest-text">{attestString}</pre>
                <CopyButton text={attestString} />
              </div>
            </li>
            <li>Paste the nevent or raw signed event JSON below</li>
          </ol>
        )}
      </div>
      <form onSubmit={handleSubmit}>
        <label htmlFor="url">{isX ? 'tweet URL' : 'nevent or raw signed event JSON'}</label>
        {isX ? (
          <input
            type="text"
            id="url"
            name="url"
            placeholder="https://x.com/.../status/..."
            required
            value={url}
            onChange={(e) => setUrl(e.target.value)}
          />
        ) : (
          <textarea
            id="url"
            name="url"
            placeholder={'nevent1... or {"pubkey":"...","sig":"...","content":"...",...}'}
            required
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            style={{ height: '80px' }}
          />
        )}
        <button type="submit" disabled={submitting}>
          {submitting ? 'verifying...' : 'verify attestation'}
        </button>
        {error && <p className="error">{error}</p>}
        {success && <p className="success">{success}</p>}
      </form>
    </div>
  );
}
