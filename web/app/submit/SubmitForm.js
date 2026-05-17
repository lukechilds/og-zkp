'use client';

import { useEffect, useRef, useState } from 'react';
import { useRouter } from 'next/navigation';

const WALLETS = {
  core: {
    label: 'bitcoin core cli',
    steps: [
      'choose an address from your bitcoin core wallet that has received bitcoin',
      'run the signmessage command below',
      'copy the signature printed by bitcoin-cli',
    ],
  },
  electrum: {
    label: 'electrum',
    steps: [
      'open the electrum wallet that controls the address you want to prove',
      'open tools and choose sign/verify message, or right click the address and choose sign/verify message',
      'paste the address and message below, sign it, then copy the signature',
    ],
  },
  sparrow: {
    label: 'sparrow',
    steps: [
      'open the wallet that controls the address you want to prove',
      'find the address in receive or addresses and open the message signing tool',
      'paste the message below, sign it, then copy the signature',
    ],
  },
  bluewallet: {
    label: 'bluewallet',
    steps: [
      'open the bitcoin wallet that controls the address you want to prove',
      'open the wallet menu and choose sign / verify message',
      'paste the address and message below, then copy the generated signature',
    ],
  },
};

function shellQuote(value) {
  return `"${value.replace(/(["\\$`])/g, '\\$1')}"`;
}

function CopyButton({ text }) {
  const [label, setLabel] = useState('copy');

  function handleCopy() {
    navigator.clipboard.writeText(text).then(() => {
      setLabel('copied');
      setTimeout(() => setLabel('copy'), 1500);
    });
  }

  return (
    <button type="button" className="copy-btn" onClick={handleCopy}>{label}</button>
  );
}

function CodeBlock({ children }) {
  return (
    <div className="code-block">
      <pre>{children}</pre>
      <CopyButton text={children} />
    </div>
  );
}

function ProofGuide({ guideRef }) {
  const [wallet, setWallet] = useState('core');
  const [identity, setIdentity] = useState('');
  const [address, setAddress] = useState('');
  const [signature, setSignature] = useState('');

  const walletConfig = WALLETS[wallet];
  const identityValue = identity.trim() || '<identity>';
  const identityWarning = identity.trim() && !identityValue.startsWith('x.com/') && !identityValue.startsWith('npub1');
  const addressValue = address.trim() || '<bitcoin-address>';
  const signatureValue = signature.trim() || '<signature-from-wallet>';
  const message = `og-zkp ${identityValue}`;
  const signCommand = `bitcoin-cli signmessage ${shellQuote(addressValue)} ${shellQuote(message)}`;
  const proveCommand = [
    'docker run -it ghcr.io/lukechilds/og-zkp prove \\',
    `  --message ${shellQuote(message)} \\`,
    `  --address ${shellQuote(addressValue)} \\`,
    `  --signature ${shellQuote(signatureValue)}`,
  ].join('\n');

  return (
    <div className="proof-guide" ref={guideRef}>
      <div className="section-title">generate a proof</div>
      <div className="guide-fields">
        <div className="field">
          <div className="label-row">
            <label htmlFor="proof-identity">identity</label>
            {identityWarning && (
              <span className="warning">should start with x.com/ or npub1</span>
            )}
          </div>
          <input
            type="text"
            id="proof-identity"
            placeholder="x.com/yourhandle or npub1..."
            value={identity}
            onChange={(e) => setIdentity(e.target.value)}
          />
        </div>
        <div className="field">
          <label htmlFor="proof-address">bitcoin address</label>
          <input
            type="text"
            id="proof-address"
            placeholder="bc1... or 1..."
            value={address}
            onChange={(e) => setAddress(e.target.value)}
          />
        </div>
        <div className="field">
          <label htmlFor="proof-wallet">wallet</label>
          <select
            id="proof-wallet"
            value={wallet}
            onChange={(e) => setWallet(e.target.value)}
          >
            {Object.entries(WALLETS).map(([key, value]) => (
              <option key={key} value={key}>{value.label}</option>
            ))}
          </select>
        </div>
      </div>

      <ol className="guide-steps">
        {walletConfig.steps.map((step) => (
          <li key={step}>{step}</li>
        ))}
      </ol>

      <div className="field">
        <div className="field-label">message to sign</div>
        <CodeBlock>{message}</CodeBlock>
      </div>

      {wallet === 'core' && (
        <div className="field">
          <div className="field-label">sign command</div>
          <CodeBlock>{signCommand}</CodeBlock>
        </div>
      )}

      <div className="field">
        <label htmlFor="proof-signature">signature</label>
        <input
          type="text"
          id="proof-signature"
          placeholder="paste wallet signature"
          value={signature}
          onChange={(e) => setSignature(e.target.value)}
        />
      </div>

      <div className="field">
        <div className="field-label">prove command</div>
        <CodeBlock>{proveCommand}</CodeBlock>
      </div>
    </div>
  );
}

export default function SubmitForm({ onSuccess }) {
  const router = useRouter();
  const [proof, setProof] = useState('');
  const [error, setError] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [showGuide, setShowGuide] = useState(false);
  const guideRef = useRef(null);

  useEffect(() => {
    if (!showGuide) return;

    let secondFrame;
    const resetScroll = () => {
      const guide = guideRef.current;
      const modal = guide?.closest('.modal');
      if (modal) {
        modal.scrollTop = 0;
      } else if (guide) {
        guide.scrollIntoView({ block: 'start' });
      }
    };

    resetScroll();
    const firstFrame = requestAnimationFrame(() => {
      resetScroll();
      secondFrame = requestAnimationFrame(resetScroll);
    });

    return () => {
      cancelAnimationFrame(firstFrame);
      if (secondFrame) cancelAnimationFrame(secondFrame);
    };
  }, [showGuide]);

  function openGuide(e) {
    e.currentTarget.blur();
    setShowGuide(true);
  }

  async function handleSubmit(e) {
    e.preventDefault();
    setError('');

    const trimmed = proof.trim();
    if (!trimmed) {
      setError('please enter a proof');
      return;
    }

    setSubmitting(true);

    try {
      const res = await fetch('/api/submit', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ proof: trimmed }),
      });
      const data = await res.json();
      if (!res.ok) {
        throw new Error(data.error || 'submission failed');
      }
      if (onSuccess) onSuccess();
      router.push(`/proof/${data.proof_id}?cache=${Date.now()}`);
    } catch (err) {
      setError(err.message.toLowerCase());
      setSubmitting(false);
    }
  }

  return (
    <>
      {showGuide && <ProofGuide guideRef={guideRef} />}

      <form onSubmit={handleSubmit}>
        <label htmlFor="proof">
          <span>paste your proof string</span>
          {!showGuide && (
            <>
              <span className="label-separator"> &mdash; </span>
              <button
                type="button"
                className="inline-link"
                onClick={openGuide}
              >
                how to generate a proof
              </button>
            </>
          )}
        </label>
        <textarea
          id="proof"
          name="proof"
          placeholder="og-zkp1..."
          required
          value={proof}
          onChange={(e) => setProof(e.target.value)}
        />
        <button type="submit" className="btn-primary" disabled={submitting}>
          {submitting ? 'verifying...' : 'submit'}
        </button>
        {error && <p className="error">{error}</p>}
      </form>
    </>
  );
}
