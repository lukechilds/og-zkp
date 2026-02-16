'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';

export default function SubmitForm() {
  const router = useRouter();
  const [proof, setProof] = useState('');
  const [error, setError] = useState('');
  const [submitting, setSubmitting] = useState(false);

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
      router.push('/proof/' + data.proof_id);
    } catch (err) {
      setError(err.message);
      setSubmitting(false);
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      <label htmlFor="proof">paste your proof string (starts with og-zkp1...)</label>
      <textarea
        id="proof"
        name="proof"
        placeholder="og-zkp1..."
        required
        value={proof}
        onChange={(e) => setProof(e.target.value)}
      />
      <button type="submit" disabled={submitting}>
        {submitting ? 'verifying...' : 'submit'}
      </button>
      {error && <p className="error">{error}</p>}
    </form>
  );
}
