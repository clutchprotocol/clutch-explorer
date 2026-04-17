export function shortHash(value: string) {
  if (!value) return "-";
  if (value.length <= 16) return value;
  return `${value.slice(0, 10)}...${value.slice(-6)}`;
}

export function formatNumber(value: number) {
  return new Intl.NumberFormat().format(value);
}

export function formatRelativeTime(value: string) {
  const date = new Date(value).getTime();
  if (Number.isNaN(date)) return value;
  const diffSeconds = Math.max(0, Math.floor((Date.now() - date) / 1000));
  if (diffSeconds < 60) return `${diffSeconds}s ago`;
  if (diffSeconds < 3600) return `${Math.floor(diffSeconds / 60)}m ago`;
  if (diffSeconds < 86400) return `${Math.floor(diffSeconds / 3600)}h ago`;
  return `${Math.floor(diffSeconds / 86400)}d ago`;
}
