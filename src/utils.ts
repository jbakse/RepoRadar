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

export function formatRelative(isoString: string): string {
  try {
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHr = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHr / 24);

    if (diffDay >= 365) {
      return `${Math.floor(diffDay / 365)}y ago`;
    }
    if (diffDay >= 7) {
      return `${Math.floor(diffDay / 7)}w ago`;
    }
    if (diffDay > 0) {
      return `${diffDay}d ago`;
    }
    if (diffHr > 0) {
      return `${diffHr}h ago`;
    }
    if (diffMin > 0) {
      return `${diffMin}m ago`;
    }
    return "just now";
  } catch {
    return isoString;
  }
}
