'use client';

import { useEffect, useRef } from 'react';

let hasVisited = false;

export default function FirstVisit({ children }) {
  const shouldAnimate = useRef(null);
  if (shouldAnimate.current === null) {
    shouldAnimate.current = !hasVisited;
  }

  useEffect(() => {
    hasVisited = true;
  }, []);

  return <div className={shouldAnimate.current ? 'first-visit' : ''}>{children}</div>;
}
