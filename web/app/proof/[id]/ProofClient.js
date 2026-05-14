'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { bech32 } from 'bech32';

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

function shellQuote(value) {
  return `"${value.replace(/(["\\$`])/g, '\\$1')}"`;
}

export default function ProofClient({ proof }) {
  const router = useRouter();
  const isVerified = proof.status === 'verified';
  const pageUrl = `https://og-zkp.com/proof/${proof.proof_id}`;
  const attestString = `I'm verifying myself on og-zkp\n\n${pageUrl}`;
  const verifyCommand = `docker run --rm ghcr.io/lukechilds/og-zkp-verifier ${shellQuote(proof.proof)}`;
  const isX = proof.identity_type === 'x';

  return (
    <>
      {isVerified && proof.attestation_url && (
        <AttestationDisplay url={proof.attestation_url} attestString={attestString} />
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

      <div className="section">
        <div className="section-title">proof</div>
        <div style={{ fontSize: '0.7rem', marginBottom: '0.75rem' }}><span style={{ color: 'var(--text-muted)' }}>id:</span> {proof.proof_id}</div>
        <div className="code-block">
          <pre>{proof.proof}</pre>
          <CopyButton text={proof.proof} />
        </div>
        <div className="local-verify">
          <div className="field-label">verify locally</div>
          <div className="code-block code-block-nowrap">
            <pre>{verifyCommand}</pre>
            <CopyButton text={verifyCommand} />
          </div>
        </div>
      </div>
    </>
  );
}

function nostrNjumpUrl(url) {
  const neventMatch = url.match(/(nevent1[a-z0-9]+)/);
  if (neventMatch) return `https://njump.me/${neventMatch[1]}`;

  const noteMatch = url.match(/(note1[a-z0-9]+)/);
  if (noteMatch) return `https://njump.me/${noteMatch[1]}`;

  try {
    const event = JSON.parse(url);
    if (event.id) {
      const bytes = event.id.match(/.{2}/g).map(b => parseInt(b, 16));
      const words = bech32.toWords(new Uint8Array(bytes));
      const note = bech32.encode('note', words, 1000);
      return `https://njump.me/${note}`;
    }
  } catch {}

  return null;
}

function AttestationDisplay({ url, attestString }) {
  const isXAttestation = url.match(/(?:x\.com|twitter\.com)\/[^/]+\/status\/(\d+)/);
  const njumpUrl = !isXAttestation ? nostrNjumpUrl(url) : null;
  const formattedUrl = (() => { try { return JSON.stringify(JSON.parse(url), null, 2); } catch { return url; } })();

  return (
    <div className="section">
      <div className="section-title">attestation</div>
      {isXAttestation ? (
        <>
          <div className="code-block">
            <pre>{attestString}</pre>
            <CopyButton text={attestString} />
          </div>
          <div className="attestation-link" style={{ marginTop: '0.75rem' }}>
            <a href={url} target="_blank" rel="noopener">{url}</a>
          </div>
        </>
      ) : (
        <>
          <div className="code-block">
            <pre>{formattedUrl}</pre>
            <CopyButton text={formattedUrl} />
          </div>
          {njumpUrl && (
            <div className="attestation-link" style={{ marginTop: '0.75rem' }}>
              <a href={njumpUrl} target="_blank" rel="noopener">{njumpUrl}</a>
            </div>
          )}
        </>
      )}
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
      setError(err.message.toLowerCase());
      setSubmitting(false);
    }
  }

  return (
    <div className="section">
      <div className="section-title">complete attestation</div>
      <div className="instructions">
        {isX ? (
          <ol>
            <li>
              post a tweet from <strong>{identity}</strong> with this exact text:
              <div className="code-block">
                <pre>{attestString}</pre>
                <CopyButton text={attestString} />
              </div>
            </li>
            <li>paste the tweet URL below</li>
          </ol>
        ) : (
          <ol>
            <li>
              sign a nostr note from your npub with this exact text:
              <div className="code-block">
                <pre>{attestString}</pre>
                <CopyButton text={attestString} />
              </div>
            </li>
            <li>paste the nevent or raw signed event JSON below</li>
          </ol>
        )}
      </div>
      <form className="attest-form" onSubmit={handleSubmit}>
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
        <button type="submit" className="btn-primary" disabled={submitting}>
          {submitting ? 'verifying...' : 'verify attestation'}
        </button>
        {error && <p className="error">{error}</p>}
        {success && <p className="success">{success}</p>}
      </form>
    </div>
  );
}
