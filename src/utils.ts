export function formatDate(isoString: string): string {
  try {
    const date = new Date(isoString);
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return isoString;
  }
}

export function formatRelative(isoString: string, compact: boolean = false): string {
  try {
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHr = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHr / 24);

    if (diffDay >= 365) {
      const n = Math.floor(diffDay / 365);
      return compact ? `${n}y` : `${n} ${n === 1 ? "year" : "years"} ago`;
    }
    if (diffDay >= 30) {
      const n = Math.floor(diffDay / 30);
      return compact ? `${n}mo` : `${n} ${n === 1 ? "month" : "months"} ago`;
    }
    if (diffDay >= 7) {
      const n = Math.floor(diffDay / 7);
      return compact ? `${n}w` : `${n} ${n === 1 ? "week" : "weeks"} ago`;
    }
    if (diffDay > 0) {
      return compact ? `${diffDay}d` : `${diffDay} ${diffDay === 1 ? "day" : "days"} ago`;
    }
    if (diffHr > 0) {
      return compact ? `${diffHr}h` : `${diffHr} ${diffHr === 1 ? "hour" : "hours"} ago`;
    }
    if (diffMin > 0) {
      return compact ? `${diffMin}m` : `${diffMin} ${diffMin === 1 ? "minute" : "minutes"} ago`;
    }
    return compact ? "now" : "just now";
  } catch {
    return isoString;
  }
}
