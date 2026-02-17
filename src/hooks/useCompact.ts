import { useState, useEffect } from "react";

const COMPACT_BREAKPOINT = 640;

export function useCompact(): boolean {
  const [isCompact, setIsCompact] = useState(
    () => window.innerWidth <= COMPACT_BREAKPOINT
  );

  useEffect(() => {
    const mql = window.matchMedia(`(max-width: ${COMPACT_BREAKPOINT}px)`);
    const handler = (e: MediaQueryListEvent) => setIsCompact(e.matches);
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  }, []);

  return isCompact;
}
