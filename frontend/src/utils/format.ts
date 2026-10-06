export function shortHash(value: string | null | undefined, head = 8, tail = 6) {
  if (!value) return "-";
  if (value.length <= head + tail + 2) return value;
  return `${value.slice(0, head)}…${value.slice(-tail)}`;
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

export function formatDateTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
  }).format(date);
}

/** Normalize hex addresses to lowercase `0x…` (RLP/on-chain often omits prefix). */
export function formatHexAddress(value: string | null | undefined): string | null {
  if (!value) return null;
  const trimmed = value.trim();
  if (!trimmed) return null;
  const body = trimmed.startsWith("0x") || trimmed.startsWith("0X") ? trimmed.slice(2) : trimmed;
  if (!body) return null;
  return `0x${body.toLowerCase()}`;
}

/// CLT is an integer micro-dollar: 1 USD = 1,000,000 CLT. Rendering the raw integer is how a
/// reader concludes the supply is a trillion of something.
export function formatClt(microDollars: number) {
  return new Intl.NumberFormat(undefined, {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 2,
  }).format(microDollars / 1_000_000);
}

/// The same, for a single amount or fee: a flat fee is 1,000 base units, which rounds to $0.00
/// at cents, so keep up to the full six decimals for anything under a dollar.
export function formatCltPrecise(microDollars: number) {
  const dollars = microDollars / 1_000_000;
  return new Intl.NumberFormat(undefined, {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: Math.abs(dollars) < 1 && dollars !== 0 ? 6 : 2,
  }).format(dollars);
}

/** Where this explorer is pointed, read from its own hostname. */
export function networkLabel(): { label: string; tone: "main" | "test" | "local" } {
  const configured = import.meta.env.VITE_NETWORK_LABEL;
  const host = typeof window !== "undefined" ? window.location.hostname : "";
  if (configured) return { label: configured, tone: /main/i.test(configured) ? "main" : "test" };
  if (host.includes("-stage.") || host.includes("stage.")) return { label: "Testnet", tone: "test" };
  if (host.endsWith("clutchprotocol.io")) return { label: "Mainnet pilot", tone: "main" };
  return { label: "Local", tone: "local" };
}
