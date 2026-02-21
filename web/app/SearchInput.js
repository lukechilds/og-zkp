'use client';

import { useRouter, useParams } from 'next/navigation';
import { useRef, useState } from 'react';

export default function SearchInput() {
  const router = useRouter();
  const params = useParams();
  const [value, setValue] = useState(params.query ? decodeURIComponent(params.query) : '');
  const timer = useRef(null);

  function handleChange(e) {
    const val = e.target.value;
    setValue(val);
    clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      router.replace(val.trim() ? `/search/${encodeURIComponent(val.trim())}` : '/');
    }, 200);
  }

  return (
    <input
      type="text"
      name="q"
      placeholder="search by identity"
      value={value}
      onChange={handleChange}
    />
  );
}
