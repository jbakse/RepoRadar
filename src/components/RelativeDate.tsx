import { useState } from "react";
import { formatRelative, formatDate } from "../utils";

export function RelativeDate({ iso, className }: { iso: string; className?: string }) {
  const [hovered, setHovered] = useState(false);
  return (
    <div
      className={className}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {hovered ? formatDate(iso) : formatRelative(iso)}
    </div>
  );
}
