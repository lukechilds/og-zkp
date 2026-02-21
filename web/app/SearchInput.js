'use client';

import { useRouter, useSearchParams } from 'next/navigation';
import { useRef } from 'react';

export default function SearchInput() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const timer = useRef(null);

  function handleChange(e) {
    clearTimeout(timer.current);
    const val = e.target.value;
    timer.current = setTimeout(() => {
      router.replace(val ? `/?q=${encodeURIComponent(val)}` : '/');
    }, 200);
  }

  return (
    <input
      type="text"
      name="q"
      placeholder="search by identity"
      defaultValue={searchParams.get('q') || ''}
      onChange={handleChange}
    />
  );
}
