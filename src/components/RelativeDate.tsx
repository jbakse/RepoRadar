import { useState } from "react";
import { formatRelative, formatDate } from "../utils";

export function RelativeDate({ iso, className, compact = false }: { iso: string; className?: string; compact?: boolean }) {
  const [hovered, setHovered] = useState(false);
  return (
    <div
      className={className}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {hovered ? formatDate(iso) : formatRelative(iso, compact)}
    </div>
  );
}
